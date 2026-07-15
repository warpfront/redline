// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::common::Measurement;
use crate::spec::{Fixture, RowSpec, TimingMode};
use anyhow::{Context, Result};
use hip_bridge::{DeviceBuffer, Function, Graph, GraphExec, HipRuntime, Module, Stream};
use radiowave::{SchedulerProfile, Wavefront};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct EmbeddedCodeObject {
    pub code: &'static [u8],
    pub manifest: &'static str,
}

macro_rules! embedded_code_object {
    ($wave:literal, $suffix:literal) => {
        EmbeddedCodeObject {
            code: include_bytes!(concat!(
                env!("OUT_DIR"),
                "/hipfire_6409_wave",
                $wave,
                $suffix,
                ".hsaco"
            )),
            manifest: include_str!(concat!(
                env!("OUT_DIR"),
                "/hipfire_6409_wave",
                $wave,
                $suffix,
                ".radiowave.json"
            )),
        }
    };
}

pub fn embedded_code_object(
    scheduler_profile: SchedulerProfile,
    wavefront: Wavefront,
) -> EmbeddedCodeObject {
    match (scheduler_profile, wavefront) {
        (SchedulerProfile::Default, Wavefront::Wave32) => embedded_code_object!("32", ""),
        (SchedulerProfile::Default, Wavefront::Wave64) => embedded_code_object!("64", ""),
        (SchedulerProfile::MaxIlp, Wavefront::Wave32) => {
            embedded_code_object!("32", "_max_ilp")
        }
        (SchedulerProfile::MaxIlp, Wavefront::Wave64) => {
            embedded_code_object!("64", "_max_ilp")
        }
        (SchedulerProfile::IterativeIlp, Wavefront::Wave32) => {
            embedded_code_object!("32", "_iterative_ilp")
        }
        (SchedulerProfile::IterativeIlp, Wavefront::Wave64) => {
            embedded_code_object!("64", "_iterative_ilp")
        }
        (SchedulerProfile::MemoryClause, Wavefront::Wave32) => {
            embedded_code_object!("32", "_memory_clause")
        }
        (SchedulerProfile::MemoryClause, Wavefront::Wave64) => {
            embedded_code_object!("64", "_memory_clause")
        }
        (SchedulerProfile::PipelineIlp, Wavefront::Wave32) => {
            embedded_code_object!("32", "_pipeline_ilp")
        }
        (SchedulerProfile::PipelineIlp, Wavefront::Wave64) => {
            embedded_code_object!("64", "_pipeline_ilp")
        }
    }
}

const KERNELS: &[&str] = &[
    "dispatch_tiny",
    "geometry_fma",
    "geometry_fma_buffer",
    "reduction_wave",
    "reduction_wave_buffer",
    "reduction_lds",
    "reduction_extra_barrier",
    "reduction_multi4",
    "reduction_multi8",
    "reduction_multi16",
    "memory_coalesced4",
    "memory_strided4",
    "memory_gather",
    "memory_interleave4",
    "memory_interleave4_buffer",
    "memory_interleave4_block64",
    "memory_interleave4_block64_b32",
    "dot_q8",
    "dot_q4",
    "dot_q6",
    "dot_scalar",
    "vopd_independent",
    "vopd_dependent",
    "vopd_mixed",
    "vopd_mixed_pair",
    "vopd_dequant",
    "vopd_dequant_chunk16",
    "sampler_argmax",
    "sampler_topk",
    "two_stage_partial",
    "two_stage_final",
    "q8_1_quantize_q4",
    "q8_1_quantize_q6",
    "q8_1_quantize_dense",
    "q4_selected_dual",
    "q6_x8",
    "dense_q8",
    "dense_q8_single",
];

pub struct HipBackend {
    pub hip: HipRuntime,
    profiles: Vec<ProfileModules>,
    pub arch: String,
}

struct ProfileModules {
    scheduler_profile: SchedulerProfile,
    _modules: [Module; 2],
    functions_wave32: HashMap<&'static str, Function>,
    functions_wave64: HashMap<&'static str, Function>,
}

pub(crate) struct Buffers {
    pub(crate) a: DeviceBuffer,
    pub(crate) b: DeviceBuffer,
    pub(crate) out: DeviceBuffer,
}

struct Events {
    start: hip_bridge::Event,
    stop: hip_bridge::Event,
    done: Vec<hip_bridge::Event>,
}

impl HipBackend {
    pub fn new() -> Result<Self> {
        let hip = HipRuntime::load().context("load Hipfire dlopen HIP bridge")?;
        hip.set_device(0)?;
        let arch = hip.get_arch(0)?;
        let mut profiles = Vec::with_capacity(SchedulerProfile::ALL.len());
        for scheduler_profile in SchedulerProfile::ALL {
            let wave32 = embedded_code_object(scheduler_profile, Wavefront::Wave32);
            let wave64 = embedded_code_object(scheduler_profile, Wavefront::Wave64);
            let module_wave32 = hip.module_load_data(wave32.code).with_context(|| {
                format!(
                    "load Hipfire wave32 {} benchmark code object",
                    scheduler_profile.as_str()
                )
            })?;
            let module_wave64 = hip.module_load_data(wave64.code).with_context(|| {
                format!(
                    "load Hipfire wave64 {} benchmark code object",
                    scheduler_profile.as_str()
                )
            })?;
            let mut functions_wave32 = HashMap::new();
            let mut functions_wave64 = HashMap::new();
            for &name in KERNELS {
                functions_wave32.insert(name, hip.module_get_function(&module_wave32, name)?);
                functions_wave64.insert(name, hip.module_get_function(&module_wave64, name)?);
            }
            profiles.push(ProfileModules {
                scheduler_profile,
                _modules: [module_wave32, module_wave64],
                functions_wave32,
                functions_wave64,
            });
        }
        Ok(Self {
            hip,
            profiles,
            arch,
        })
    }

    pub(crate) fn allocate(
        &self,
        spec: &RowSpec,
        fixture: &Fixture,
        mode: TimingMode,
    ) -> Result<Buffers> {
        let a = self.hip.malloc(fixture.a.len().max(1) * 4)?;
        let b = self.hip.malloc(fixture.b.len().max(1) * 4)?;
        let out = self.hip.malloc(spec.output_words(mode).max(1) * 4)?;
        self.hip.memcpy_htod(&a, words_as_bytes(&fixture.a))?;
        self.hip.memcpy_htod(&b, words_as_bytes(&fixture.b))?;
        self.hip.memset(&out, 0, out.size())?;
        self.hip.device_synchronize()?;
        Ok(Buffers { a, b, out })
    }

    fn make_events(&self, lanes: usize) -> Result<Events> {
        Ok(Events {
            start: self.hip.event_create()?,
            stop: self.hip.event_create()?,
            done: (0..lanes)
                .map(|_| self.hip.event_create())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the explicit launch ABI.
    fn function(&self, name: &str, spec: &RowSpec) -> Result<&Function> {
        let profile = self
            .profiles
            .iter()
            .find(|profile| profile.scheduler_profile == spec.scheduler_profile)
            .with_context(|| {
                format!(
                    "missing HIP scheduler profile {}",
                    spec.scheduler_profile.as_str()
                )
            })?;
        let functions = match spec.wave_size {
            32 => &profile.functions_wave32,
            64 => &profile.functions_wave64,
            other => anyhow::bail!("unsupported HIP wave size {other}"),
        };
        functions
            .get(name)
            .with_context(|| format!("missing HIP wave{} kernel {name}", spec.wave_size))
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the explicit launch ABI.
    fn launch(
        &self,
        name: &str,
        grid_groups: u32,
        spec: &RowSpec,
        offset: u32,
        second: bool,
        buffers: &Buffers,
        stream: &Stream,
        blob: &mut [u8],
    ) -> Result<()> {
        fill_blob(blob, buffers, spec, offset, second);
        let function = self.function(name, spec)?;
        let block = spec.stage_block(second);
        unsafe {
            self.hip.launch_kernel_blob(
                function,
                [grid_groups * block, 1, 1],
                [block, 1, 1],
                0,
                Some(stream),
                blob,
            )?;
        }
        Ok(())
    }

    fn launch_operation(
        &self,
        spec: &RowSpec,
        mode: TimingMode,
        operation: usize,
        buffers: &Buffers,
        stream: &Stream,
        blobs: &mut Vec<Vec<u8>>,
    ) -> Result<()> {
        let offset = spec.stage_output_offset(mode, operation, false);
        blobs.push(vec![0u8; 40]);
        self.launch(
            spec.kernel,
            spec.grid_groups,
            spec,
            offset,
            false,
            buffers,
            stream,
            blobs.last_mut().unwrap(),
        )?;
        if let Some(second) = spec.second_kernel {
            let second_offset = spec.stage_output_offset(mode, operation, true);
            blobs.push(vec![0u8; 40]);
            self.launch(
                second,
                spec.second_grid_groups,
                spec,
                second_offset,
                true,
                buffers,
                stream,
                blobs.last_mut().unwrap(),
            )?;
        }
        Ok(())
    }

    pub(crate) fn reset(&self, out: &DeviceBuffer) -> Result<()> {
        self.hip.memset(out, 0, out.size())?;
        self.hip.device_synchronize()?;
        Ok(())
    }

    fn time_direct_once(
        &self,
        spec: &RowSpec,
        mode: TimingMode,
        buffers: &Buffers,
        streams: &[Stream],
        coordinator: &Stream,
        events: &Events,
    ) -> Result<f64> {
        self.reset(&buffers.out)?;
        let lanes = streams.len();
        let iterations = spec.logical_iterations(mode);
        if lanes == 1 {
            self.hip.event_record(&events.start, Some(&streams[0]))?;
            let mut blobs = Vec::new();
            for op in 0..iterations {
                self.launch_operation(spec, mode, op, buffers, &streams[0], &mut blobs)?;
            }
            self.hip.event_record(&events.stop, Some(&streams[0]))?;
        } else {
            self.hip.event_record(&events.start, Some(coordinator))?;
            for stream in streams {
                self.hip.stream_wait_event(stream, &events.start)?;
            }
            let mut lane_blobs = (0..lanes).map(|_| Vec::new()).collect::<Vec<_>>();
            for op in 0..iterations {
                let lane = op % lanes;
                self.launch_operation(
                    spec,
                    mode,
                    op,
                    buffers,
                    &streams[lane],
                    &mut lane_blobs[lane],
                )?;
            }
            for (lane, stream) in streams.iter().enumerate() {
                self.hip.event_record(&events.done[lane], Some(stream))?;
                self.hip
                    .stream_wait_event(coordinator, &events.done[lane])?;
            }
            self.hip.event_record(&events.stop, Some(coordinator))?;
        }
        self.hip.event_synchronize(&events.stop)?;
        Ok(
            self.hip.event_elapsed_ms(&events.start, &events.stop)? as f64 * 1000.0
                / iterations as f64,
        )
    }

    pub fn measure_direct(
        &self,
        spec: &RowSpec,
        fixture: &Fixture,
        mode: TimingMode,
        warmups: usize,
        samples: usize,
    ) -> Result<Measurement> {
        let buffers = self.allocate(spec, fixture, mode)?;
        let lanes = if mode == TimingMode::IndependentThroughput {
            4.min(spec.logical_iterations(mode))
        } else {
            1
        };
        let streams = (0..lanes)
            .map(|_| self.hip.stream_create())
            .collect::<Result<Vec<_>, _>>()?;
        let coordinator = self.hip.stream_create()?;
        let events = self.make_events(lanes)?;
        for _ in 0..warmups {
            self.time_direct_once(spec, mode, &buffers, &streams, &coordinator, &events)?;
        }
        let mut gpu_samples_us = Vec::with_capacity(samples);
        for _ in 0..samples {
            gpu_samples_us.push(self.time_direct_once(
                spec,
                mode,
                &buffers,
                &streams,
                &coordinator,
                &events,
            )?);
        }
        let output = self.read_output(&buffers.out, spec.output_words(mode))?;
        self.destroy_events(events)?;
        for stream in streams {
            self.hip.stream_destroy(stream)?;
        }
        self.hip.stream_destroy(coordinator)?;
        self.free_buffers(buffers)?;
        Ok(Measurement {
            gpu_samples_us,
            output,
        })
    }

    pub fn measure_graph(
        &self,
        spec: &RowSpec,
        fixture: &Fixture,
        mode: TimingMode,
        warmups: usize,
        samples: usize,
    ) -> Result<Measurement> {
        // Fail before beginning stream capture when a Radiowave-selected
        // variant is absent. Returning from inside capture leaves HIP's stream
        // capture state active and poisons every later backend allocation.
        self.function(spec.kernel, spec)?;
        if let Some(second) = spec.second_kernel {
            self.function(second, spec)?;
        }
        let buffers = self.allocate(spec, fixture, mode)?;
        let lanes = if mode == TimingMode::IndependentThroughput {
            4.min(spec.logical_iterations(mode))
        } else {
            1
        };
        let streams = (0..lanes)
            .map(|_| self.hip.stream_create())
            .collect::<Result<Vec<_>, _>>()?;
        let coordinator = self.hip.stream_create()?;
        let events = self.make_events(lanes)?;
        let mut persistent_blobs = (0..lanes)
            .map(|_| Vec::new())
            .collect::<Vec<Vec<Vec<u8>>>>();
        let mut graphs: Vec<Graph> = Vec::with_capacity(lanes);
        let mut execs: Vec<GraphExec> = Vec::with_capacity(lanes);
        for lane in 0..lanes {
            self.hip.stream_begin_capture(&streams[lane], 0)?;
            for op in (lane..spec.logical_iterations(mode)).step_by(lanes) {
                self.launch_operation(
                    spec,
                    mode,
                    op,
                    &buffers,
                    &streams[lane],
                    &mut persistent_blobs[lane],
                )?;
            }
            let graph = self.hip.stream_end_capture(&streams[lane])?;
            let exec = self.hip.graph_instantiate(&graph)?;
            graphs.push(graph);
            execs.push(exec);
        }

        let time_once = |this: &Self| -> Result<f64> {
            this.reset(&buffers.out)?;
            if lanes == 1 {
                this.hip.event_record(&events.start, Some(&streams[0]))?;
                this.hip.graph_launch(&execs[0], &streams[0])?;
                this.hip.event_record(&events.stop, Some(&streams[0]))?;
            } else {
                this.hip.event_record(&events.start, Some(&coordinator))?;
                for stream in &streams {
                    this.hip.stream_wait_event(stream, &events.start)?;
                }
                for lane in 0..lanes {
                    this.hip.graph_launch(&execs[lane], &streams[lane])?;
                    this.hip
                        .event_record(&events.done[lane], Some(&streams[lane]))?;
                    this.hip
                        .stream_wait_event(&coordinator, &events.done[lane])?;
                }
                this.hip.event_record(&events.stop, Some(&coordinator))?;
            }
            this.hip.event_synchronize(&events.stop)?;
            Ok(
                this.hip.event_elapsed_ms(&events.start, &events.stop)? as f64 * 1000.0
                    / spec.logical_iterations(mode) as f64,
            )
        };
        for _ in 0..warmups {
            time_once(self)?;
        }
        let mut gpu_samples_us = Vec::with_capacity(samples);
        for _ in 0..samples {
            gpu_samples_us.push(time_once(self)?);
        }
        let output = self.read_output(&buffers.out, spec.output_words(mode))?;
        for exec in execs {
            self.hip.graph_exec_destroy(exec)?;
        }
        for graph in graphs {
            self.hip.graph_destroy(graph)?;
        }
        drop(persistent_blobs);
        self.destroy_events(events)?;
        for stream in streams {
            self.hip.stream_destroy(stream)?;
        }
        self.hip.stream_destroy(coordinator)?;
        self.free_buffers(buffers)?;
        Ok(Measurement {
            gpu_samples_us,
            output,
        })
    }

    pub(crate) fn read_output(&self, out: &DeviceBuffer, words: usize) -> Result<Vec<u32>> {
        let mut bytes = vec![0u8; words * 4];
        self.hip.memcpy_dtoh(&mut bytes, out)?;
        Ok(bytes
            .chunks_exact(4)
            .map(|v| u32::from_ne_bytes(v.try_into().unwrap()))
            .collect())
    }

    fn destroy_events(&self, events: Events) -> Result<()> {
        self.hip.event_destroy(events.start)?;
        self.hip.event_destroy(events.stop)?;
        for event in events.done {
            self.hip.event_destroy(event)?;
        }
        Ok(())
    }

    pub(crate) fn free_buffers(&self, buffers: Buffers) -> Result<()> {
        self.hip.free(buffers.a)?;
        self.hip.free(buffers.b)?;
        self.hip.free(buffers.out)?;
        Ok(())
    }
}

fn words_as_bytes(words: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), words.len() * 4) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_function_table_covers_radiowave_builtin_variants() {
        for variant in [
            "geometry_fma_buffer",
            "reduction_wave_buffer",
            "memory_interleave4_buffer",
            "memory_interleave4_block64",
            "memory_interleave4_block64_b32",
            "vopd_dequant_chunk16",
            "vopd_mixed_pair",
        ] {
            assert!(KERNELS.contains(&variant), "missing HIP kernel {variant}");
        }
    }
}

fn fill_blob(blob: &mut [u8], buffers: &Buffers, spec: &RowSpec, offset: u32, second: bool) {
    blob.fill(0);
    let a = buffers.a.as_ptr() as usize as u64;
    let b = buffers.b.as_ptr() as usize as u64;
    let out = buffers.out.as_ptr() as usize as u64;
    blob[0..8].copy_from_slice(&a.to_ne_bytes());
    blob[8..16].copy_from_slice(&b.to_ne_bytes());
    blob[16..24].copy_from_slice(&out.to_ne_bytes());
    blob[24..28].copy_from_slice(&spec.n0.to_ne_bytes());
    blob[28..32].copy_from_slice(&spec.stage_n1(second).to_ne_bytes());
    blob[32..36].copy_from_slice(&offset.to_ne_bytes());
    blob[36..40].copy_from_slice(&spec.stage_aux(second).to_ne_bytes());
}
