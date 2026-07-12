// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

//! Migrating a HipGraph (or a re-submitted Vulkan command buffer) to Redline.
//!
//! Run with: `cargo run -p redline-dispatch --example hipgraph_migration`
//!
//! The shape is identical to HipGraph — create, add kernel nodes with their
//! dependencies, instantiate, launch repeatedly — with one addition: each node
//! declares the buffer regions it reads and writes, which is what lets Redline
//! derive minimal correct fences instead of a blanket per-dispatch fence.

use redline_dispatch::hipgraph::{Graph, Tuning};
use redline_dispatch::mock::MockBackend;
use redline_dispatch::{Access, Dim3, KernelLaunch, ReplayToken};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // hipGraphCreate(...) — plus a tuning choice HipGraph doesn't expose.
    // Default (Tuning::latency) replays like a single serial stream; switch to
    // `Tuning::overlap(n)` or `Tuning::throughput(n, k)` to let independent
    // branches run concurrently.
    let mut graph = Graph::with_tuning(Tuning::latency());

    // The buffers your kernel nodes touch (label + byte size).
    let acts = graph.buffer("activations", 4096)?;
    let input = graph.region(acts, 0, 2048)?;
    let output = graph.region(acts, 2048, 2048)?;

    // hipGraphAddKernelNode(project, graph, /*deps*/ none, params)
    // — the [Access] list is the one thing you add vs HipGraph.
    let project = graph.kernel(
        KernelLaunch::new("project", Dim3::x(32)?, Dim3::x(64)?)?,
        [Access::read(input), Access::write(output)],
    )?;

    // hipGraphAddKernelNode(consume, graph, /*deps*/ [project], params)
    let _consume = graph.kernel_after(
        KernelLaunch::new("consume", Dim3::x(32)?, Dim3::x(64)?)?,
        [Access::read(output)],
        [project],
    )?;

    // hipGraphInstantiate(&exec, graph, ...) — hazards validated here.
    let exec = graph.instantiate()?;
    let plan = exec.plan();
    println!(
        "instantiated: {} lane(s), fingerprint {:?}",
        plan.lane_count(),
        plan.fingerprint(),
    );

    // hipGraphLaunch(exec, stream) — replay is allocation-free; loop it.
    let mut backend = MockBackend::default();
    for i in 0..3 {
        exec.launch(&mut backend, ReplayToken(0))?;
        println!("launch {i} complete");
    }

    Ok(())
}
