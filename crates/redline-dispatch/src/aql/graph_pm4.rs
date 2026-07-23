// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Lower a compiled dispatch plan to one retained, architecture-specific PM4 IB.

use std::fmt;

use redline_rocr::{
    Gfx10Pm4BuildError, Gfx10Pm4CommandBuffer, Gfx12Pm4CommandBuffer, GpuDevice, KernargBuffer,
    KernargPool, Kernel, LaunchGeometry, PacketError, Pm4BuildError, RuntimeError,
};

use super::{ReplayError, SingleQueuePm4Ib};
use crate::{CompiledPlan, NodeId};

/// Concrete launch data bound to one graph node.
#[derive(Clone, Copy)]
pub struct NodeDispatch<'a> {
    pub kernel: &'a Kernel,
    pub kernargs: &'a [u8],
    /// Global work-item dimensions.
    pub grid: [u32; 3],
    pub block: [u16; 3],
    pub dyn_group: u32,
}

/// A retained PM4 graph replay that owns every allocation and kernel required
/// by the encoded indirect buffer.
pub struct Pm4GraphReplay {
    ib: SingleQueuePm4Ib,
    _kernargs: Vec<KernargBuffer>,
    _kernels: Vec<Kernel>,
}

impl Pm4GraphReplay {
    /// Submit the retained IB once and wait for completion.
    ///
    /// # Safety
    ///
    /// Device pointers embedded in the retained kernarg bytes must remain live
    /// and GPU-accessible until this method returns. After an error, they must
    /// remain live through destruction of this replay object.
    pub unsafe fn replay_and_wait(&mut self) -> Result<(), GraphPm4Error> {
        // SAFETY: forwarded from this method's caller. Kernarg allocations and
        // code objects are retained by this object.
        unsafe { self.ib.replay_and_wait() }.map_err(GraphPm4Error::Replay)
    }
}

/// Failure while resolving, encoding, or replaying a compiled PM4 graph.
#[derive(Debug)]
pub enum GraphPm4Error {
    UnsupportedArchitecture { actual: String },
    MissingNode(NodeId),
    Runtime(RuntimeError),
    Geometry(PacketError),
    Gfx10Build(Gfx10Pm4BuildError),
    Gfx12Build(Pm4BuildError),
    Replay(ReplayError),
}

impl fmt::Display for GraphPm4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedArchitecture { actual } => {
                write!(
                    f,
                    "PM4 graph replay does not support device architecture {actual}"
                )
            }
            Self::MissingNode(node) => write!(f, "no PM4 dispatch binding for {node:?}"),
            Self::Runtime(error) => write!(f, "PM4 graph kernarg allocation failed: {error}"),
            Self::Geometry(error) => write!(f, "invalid PM4 graph launch geometry: {error}"),
            Self::Gfx10Build(error) => write!(f, "GFX10/GFX11 PM4 graph encoding failed: {error}"),
            Self::Gfx12Build(error) => write!(f, "GFX12 PM4 graph encoding failed: {error}"),
            Self::Replay(error) => write!(f, "PM4 graph replay failed: {error}"),
        }
    }
}

impl std::error::Error for GraphPm4Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Geometry(error) => Some(error),
            Self::Gfx10Build(error) => Some(error),
            Self::Gfx12Build(error) => Some(error),
            Self::Replay(error) => Some(error),
            Self::UnsupportedArchitecture { .. } | Self::MissingNode(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pm4Family {
    Gfx10,
    Gfx11,
    Gfx12,
}

impl Pm4Family {
    fn from_name(name: &str) -> Option<Self> {
        if name.starts_with("gfx10") {
            Some(Self::Gfx10)
        } else if name.starts_with("gfx11") {
            Some(Self::Gfx11)
        } else if name.starts_with("gfx12") {
            Some(Self::Gfx12)
        } else {
            None
        }
    }
}

enum Pm4Commands {
    Legacy(Gfx10Pm4CommandBuffer),
    Gfx12(Gfx12Pm4CommandBuffer),
}

impl Pm4Commands {
    fn stateful(family: Pm4Family) -> Self {
        match family {
            Pm4Family::Gfx10 | Pm4Family::Gfx11 => {
                Self::Legacy(Gfx10Pm4CommandBuffer::new_stateful())
            }
            Pm4Family::Gfx12 => Self::Gfx12(Gfx12Pm4CommandBuffer::new_stateful()),
        }
    }

    fn dependency_boundary(&mut self) {
        match self {
            Self::Legacy(commands) => commands.dependency_rmw_same_agent(),
            Self::Gfx12(commands) => commands.dependency_rmw_same_agent_gfx12(),
        }
    }

    fn dispatch(
        &mut self,
        kernel: &Kernel,
        geometry: LaunchGeometry,
        dyn_group: u32,
        kernarg: &KernargBuffer,
    ) -> Result<(), GraphPm4Error> {
        match self {
            Self::Legacy(commands) => commands
                .dispatch(kernel, geometry, dyn_group, kernarg.address())
                .map_err(GraphPm4Error::Gfx10Build),
            Self::Gfx12(commands) => commands
                .dispatch(kernel, geometry, dyn_group, kernarg.address())
                .map_err(GraphPm4Error::Gfx12Build),
        }
    }
}

/// Lower `plan` in dispatch-slice order to one retained PM4 indirect buffer.
///
/// A same-agent RMW dependency boundary is emitted immediately before every
/// non-root dispatch whose compiled dependency list is non-empty. This uses the
/// plan's actual dependency information rather than conservatively fencing
/// every adjacent pair.
pub fn lower_plan_to_pm4_ib<'a>(
    device: &GpuDevice,
    pool: &KernargPool,
    plan: &CompiledPlan,
    mut resolve: impl FnMut(NodeId) -> Option<NodeDispatch<'a>>,
) -> Result<Pm4GraphReplay, GraphPm4Error> {
    let family = Pm4Family::from_name(device.name()).ok_or_else(|| {
        GraphPm4Error::UnsupportedArchitecture {
            actual: device.name().to_owned(),
        }
    })?;
    let mut commands = Pm4Commands::stateful(family);
    let mut kernargs = Vec::with_capacity(plan.dispatches().len());
    let mut kernels = Vec::with_capacity(plan.dispatches().len());

    for (index, dispatch) in plan.dispatches().iter().enumerate() {
        if index > 0 && !dispatch.dependencies().is_empty() {
            commands.dependency_boundary();
        }
        let binding =
            resolve(dispatch.node()).ok_or(GraphPm4Error::MissingNode(dispatch.node()))?;
        let geometry =
            LaunchGeometry::new(binding.grid, binding.block).map_err(GraphPm4Error::Geometry)?;
        let mut kernarg = pool
            .allocate_for(binding.kernel.metadata())
            .map_err(GraphPm4Error::Runtime)?;
        {
            let destination = kernarg.as_mut_bytes();
            destination.fill(0);
            let bytes = binding.kernargs.len().min(destination.len());
            destination[..bytes].copy_from_slice(&binding.kernargs[..bytes]);
        }
        commands.dispatch(binding.kernel, geometry, binding.dyn_group, &kernarg)?;
        kernels.push(binding.kernel.clone());
        kernargs.push(kernarg);
    }

    let ib = match (family, &commands) {
        (Pm4Family::Gfx10, Pm4Commands::Legacy(commands)) => {
            SingleQueuePm4Ib::create_gfx10(device, pool, commands)
        }
        (Pm4Family::Gfx11, Pm4Commands::Legacy(commands)) => {
            SingleQueuePm4Ib::create_gfx11(device, pool, commands)
        }
        (Pm4Family::Gfx12, Pm4Commands::Gfx12(commands)) => {
            SingleQueuePm4Ib::create(device, pool, commands)
        }
        _ => unreachable!("PM4 command family is selected from the same device family"),
    }
    .map_err(GraphPm4Error::Replay)?;

    Ok(Pm4GraphReplay {
        ib,
        _kernargs: kernargs,
        _kernels: kernels,
    })
}

#[cfg(test)]
mod tests {
    use super::Pm4Family;

    #[test]
    fn rdna_generations_select_their_pm4_family() {
        assert_eq!(Pm4Family::from_name("gfx1010"), Some(Pm4Family::Gfx10));
        assert_eq!(Pm4Family::from_name("gfx1030"), Some(Pm4Family::Gfx10));
        assert_eq!(Pm4Family::from_name("gfx1100"), Some(Pm4Family::Gfx11));
        assert_eq!(Pm4Family::from_name("gfx1201"), Some(Pm4Family::Gfx12));
        assert_eq!(Pm4Family::from_name("gfx900"), None);
    }
}
