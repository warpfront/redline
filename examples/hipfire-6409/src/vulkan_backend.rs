// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use crate::common::Measurement;
use crate::spec::{Fixture, RowSpec, TimingMode};
use anyhow::{bail, Context, Result};
use ash::{vk, Entry};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::io::Cursor;

const SHADERS: &[(&str, &[u8])] = &[
    (
        "dispatch_tiny",
        include_bytes!(concat!(env!("OUT_DIR"), "/dispatch_tiny.spv")),
    ),
    (
        "geometry_fma",
        include_bytes!(concat!(env!("OUT_DIR"), "/geometry_fma.spv")),
    ),
    (
        "reduction_wave",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_wave.spv")),
    ),
    (
        "reduction_lds",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_lds.spv")),
    ),
    (
        "reduction_extra_barrier",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_extra_barrier.spv")),
    ),
    (
        "reduction_multi4",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_multi4.spv")),
    ),
    (
        "reduction_multi8",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_multi8.spv")),
    ),
    (
        "reduction_multi16",
        include_bytes!(concat!(env!("OUT_DIR"), "/reduction_multi16.spv")),
    ),
    (
        "memory_coalesced4",
        include_bytes!(concat!(env!("OUT_DIR"), "/memory_coalesced4.spv")),
    ),
    (
        "memory_strided4",
        include_bytes!(concat!(env!("OUT_DIR"), "/memory_strided4.spv")),
    ),
    (
        "memory_gather",
        include_bytes!(concat!(env!("OUT_DIR"), "/memory_gather.spv")),
    ),
    (
        "memory_interleave4",
        include_bytes!(concat!(env!("OUT_DIR"), "/memory_interleave4.spv")),
    ),
    (
        "dot_q8",
        include_bytes!(concat!(env!("OUT_DIR"), "/dot_q8.spv")),
    ),
    (
        "dot_q4",
        include_bytes!(concat!(env!("OUT_DIR"), "/dot_q4.spv")),
    ),
    (
        "dot_q6",
        include_bytes!(concat!(env!("OUT_DIR"), "/dot_q6.spv")),
    ),
    (
        "dot_scalar",
        include_bytes!(concat!(env!("OUT_DIR"), "/dot_scalar.spv")),
    ),
    (
        "vopd_independent",
        include_bytes!(concat!(env!("OUT_DIR"), "/vopd_independent.spv")),
    ),
    (
        "vopd_dependent",
        include_bytes!(concat!(env!("OUT_DIR"), "/vopd_dependent.spv")),
    ),
    (
        "vopd_mixed",
        include_bytes!(concat!(env!("OUT_DIR"), "/vopd_mixed.spv")),
    ),
    (
        "vopd_dequant",
        include_bytes!(concat!(env!("OUT_DIR"), "/vopd_dequant.spv")),
    ),
    (
        "sampler_argmax",
        include_bytes!(concat!(env!("OUT_DIR"), "/sampler_argmax.spv")),
    ),
    (
        "sampler_topk",
        include_bytes!(concat!(env!("OUT_DIR"), "/sampler_topk.spv")),
    ),
    (
        "two_stage_partial",
        include_bytes!(concat!(env!("OUT_DIR"), "/two_stage_partial.spv")),
    ),
    (
        "two_stage_final",
        include_bytes!(concat!(env!("OUT_DIR"), "/two_stage_final.spv")),
    ),
    (
        "q8_1_quantize_q4",
        include_bytes!(concat!(env!("OUT_DIR"), "/q8_1_quantize_q4.spv")),
    ),
    (
        "q8_1_quantize_q6",
        include_bytes!(concat!(env!("OUT_DIR"), "/q8_1_quantize_q6.spv")),
    ),
    (
        "q8_1_quantize_dense",
        include_bytes!(concat!(env!("OUT_DIR"), "/q8_1_quantize_dense.spv")),
    ),
    (
        "q4_selected_dual",
        include_bytes!(concat!(env!("OUT_DIR"), "/q4_selected_dual.spv")),
    ),
    (
        "q6_x8",
        include_bytes!(concat!(env!("OUT_DIR"), "/q6_x8.spv")),
    ),
    (
        "dense_q8",
        include_bytes!(concat!(env!("OUT_DIR"), "/dense_q8.spv")),
    ),
    (
        "dense_q8_single",
        include_bytes!(concat!(env!("OUT_DIR"), "/dense_q8_single.spv")),
    ),
];

struct GpuBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

struct Descriptor {
    pool: vk::DescriptorPool,
    set: vk::DescriptorSet,
}

struct Lane {
    command: vk::CommandBuffer,
    query: vk::QueryPool,
    fence: vk::Fence,
}

pub struct VulkanBackend {
    _entry: Entry,
    instance: ash::Instance,
    device: ash::Device,
    queues: Vec<vk::Queue>,
    command_pool: vk::CommandPool,
    descriptor_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipelines: HashMap<(&'static str, u32), vk::Pipeline>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    timestamp_period_ns: f64,
    timestamp_valid_bits: u32,
    pub name: String,
    pub pci: Option<String>,
    pub queue_count: usize,
}

impl VulkanBackend {
    pub fn new() -> Result<Self> {
        let entry = unsafe { Entry::load() }.context("load Vulkan loader")?;
        let app_name = CString::new("hipfire-6409")?;
        let app = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(&app_name)
            .engine_version(1)
            .api_version(vk::API_VERSION_1_3);
        let instance_info = vk::InstanceCreateInfo::default().application_info(&app);
        let instance = unsafe { entry.create_instance(&instance_info, None) }
            .context("create Vulkan instance")?;
        let physicals = unsafe { instance.enumerate_physical_devices() }?;
        let physical = physicals
            .iter()
            .copied()
            .find(|&p| unsafe { instance.get_physical_device_properties(p) }.vendor_id == 0x1002)
            .context("no AMD Vulkan physical device")?;
        let properties = unsafe { instance.get_physical_device_properties(physical) };
        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let families = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let (queue_family, family) = families
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.queue_flags.contains(vk::QueueFlags::COMPUTE) && f.timestamp_valid_bits > 0
            })
            .max_by_key(|(_, f)| f.queue_count)
            .map(|(i, f)| (i as u32, *f))
            .context("no timestamp-capable Vulkan compute queue")?;
        let queue_count = usize::min(4, family.queue_count as usize);
        let priorities = vec![1.0f32; queue_count];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family)
            .queue_priorities(&priorities)];

        let mut supported_dot = vk::PhysicalDeviceShaderIntegerDotProductFeatures::default();
        let mut features2 = vk::PhysicalDeviceFeatures2::default().push_next(&mut supported_dot);
        unsafe { instance.get_physical_device_features2(physical, &mut features2) };
        if supported_dot.shader_integer_dot_product == 0 {
            unsafe { instance.destroy_instance(None) };
            bail!("selected Vulkan device lacks shaderIntegerDotProduct");
        }
        let mut dot = vk::PhysicalDeviceShaderIntegerDotProductFeatures::default()
            .shader_integer_dot_product(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .push_next(&mut dot);
        let device = unsafe { instance.create_device(physical, &device_info, None) }
            .context("create Vulkan compute device")?;
        let queues = (0..queue_count)
            .map(|i| unsafe { device.get_device_queue(queue_family, i as u32) })
            .collect::<Vec<_>>();
        let command_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(queue_family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        }?;
        let bindings = [0u32, 1, 2].map(|binding| {
            vk::DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE)
        });
        let descriptor_layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )
        }?;
        let push = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(16)];
        let set_layouts = [descriptor_layout];
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&push),
                None,
            )
        }?;
        let entry_name = CString::new("main")?;
        let mut pipelines = HashMap::new();
        for &(shader_name, bytes) in SHADERS {
            let words = ash::util::read_spv(&mut Cursor::new(bytes))?;
            let module = unsafe {
                device
                    .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)
            }?;
            for block in [32u32, 64, 128, 256] {
                let entries = [vk::SpecializationMapEntry {
                    constant_id: 0,
                    offset: 0,
                    size: std::mem::size_of::<u32>(),
                }];
                let data = block.to_ne_bytes();
                let specialization = vk::SpecializationInfo::default()
                    .map_entries(&entries)
                    .data(&data);
                let stage = vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .module(module)
                    .name(&entry_name)
                    .specialization_info(&specialization);
                let info = [vk::ComputePipelineCreateInfo::default()
                    .stage(stage)
                    .layout(pipeline_layout)];
                let created = unsafe {
                    device.create_compute_pipelines(vk::PipelineCache::null(), &info, None)
                }
                .map_err(|(_, e)| e)?;
                pipelines.insert((shader_name, block), created[0]);
            }
            unsafe { device.destroy_shader_module(module, None) };
        }
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(physical) };
        let mut pci_info = vk::PhysicalDevicePCIBusInfoPropertiesEXT::default();
        let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut pci_info);
        unsafe { instance.get_physical_device_properties2(physical, &mut props2) };
        let pci = (pci_info.pci_domain != 0 || pci_info.pci_bus != 0 || pci_info.pci_device != 0)
            .then(|| {
                format!(
                    "{:04x}:{:02x}:{:02x}.{}",
                    pci_info.pci_domain,
                    pci_info.pci_bus,
                    pci_info.pci_device,
                    pci_info.pci_function
                )
            });
        Ok(Self {
            _entry: entry,
            instance,
            device,
            queues,
            command_pool,
            descriptor_layout,
            pipeline_layout,
            pipelines,
            memory_properties,
            timestamp_period_ns: properties.limits.timestamp_period as f64,
            timestamp_valid_bits: family.timestamp_valid_bits,
            name,
            pci,
            queue_count,
        })
    }

    pub fn measure(
        &self,
        spec: &RowSpec,
        fixture: &Fixture,
        mode: TimingMode,
        warmups: usize,
        samples: usize,
    ) -> Result<Measurement> {
        let a = self.create_device_buffer((fixture.a.len().max(1) * 4) as u64)?;
        let b = self.create_device_buffer((fixture.b.len().max(1) * 4) as u64)?;
        let out = self.create_device_buffer((spec.output_words(mode).max(1) * 4) as u64)?;
        self.upload(&a, words_as_bytes(&fixture.a))?;
        self.upload(&b, words_as_bytes(&fixture.b))?;
        let descriptor = self.create_descriptor(&a, &b, &out)?;
        let iterations = spec.logical_iterations(mode);
        let lane_count = if mode == TimingMode::IndependentThroughput {
            self.queue_count.min(iterations)
        } else {
            1
        };
        let lanes = self.build_lanes(spec, mode, lane_count, descriptor.set)?;
        for _ in 0..warmups {
            self.run_once(&lanes, &out, iterations)?;
        }
        let mut gpu_samples_us = Vec::with_capacity(samples);
        for _ in 0..samples {
            gpu_samples_us.push(self.run_once(&lanes, &out, iterations)?);
        }
        let output = self.download_words(&out, spec.output_words(mode))?;
        self.destroy_lanes(lanes);
        unsafe {
            self.device.destroy_descriptor_pool(descriptor.pool, None);
            self.destroy_buffer(out);
            self.destroy_buffer(b);
            self.destroy_buffer(a);
        }
        Ok(Measurement {
            gpu_samples_us,
            output,
        })
    }

    fn build_lanes(
        &self,
        spec: &RowSpec,
        mode: TimingMode,
        lane_count: usize,
        descriptor: vk::DescriptorSet,
    ) -> Result<Vec<Lane>> {
        let commands = unsafe {
            self.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(lane_count as u32),
            )
        }?;
        let mut lanes = Vec::with_capacity(lane_count);
        for (lane_index, &command) in commands.iter().enumerate() {
            let query = unsafe {
                self.device.create_query_pool(
                    &vk::QueryPoolCreateInfo::default()
                        .query_type(vk::QueryType::TIMESTAMP)
                        .query_count(2),
                    None,
                )
            }?;
            let fence = unsafe {
                self.device
                    .create_fence(&vk::FenceCreateInfo::default(), None)
            }?;
            unsafe {
                self.device.begin_command_buffer(
                    command,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE),
                )?;
                self.device.cmd_reset_query_pool(command, query, 0, 2);
                self.device.cmd_write_timestamp(
                    command,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    query,
                    0,
                );
                self.device.cmd_bind_descriptor_sets(
                    command,
                    vk::PipelineBindPoint::COMPUTE,
                    self.pipeline_layout,
                    0,
                    &[descriptor],
                    &[],
                );
                let iterations = spec.logical_iterations(mode);
                for operation in (lane_index..iterations).step_by(lane_count) {
                    let offset = spec.output_offset(mode, operation);
                    let (kernel, grid_groups) = if matches!(
                        spec.kernel,
                        "memory_interleave4_block64" | "memory_interleave4_block64_b32"
                    ) {
                        ("memory_interleave4", spec.n0.div_ceil(256))
                    } else if spec.kernel == "memory_interleave4_buffer" {
                        ("memory_interleave4", spec.grid_groups)
                    } else if spec.kernel == "vopd_dequant_chunk16" {
                        ("vopd_dequant", spec.grid_groups)
                    } else if spec.kernel == "vopd_mixed_pair" {
                        ("vopd_mixed", spec.grid_groups)
                    } else {
                        (spec.kernel, spec.grid_groups)
                    };
                    self.record_dispatch(command, kernel, grid_groups, spec, offset, false)?;
                    if let Some(second) = spec.second_kernel {
                        self.record_dependency(command);
                        self.record_dispatch(
                            command,
                            second,
                            spec.second_grid_groups,
                            spec,
                            spec.stage_output_offset(mode, operation, true),
                            true,
                        )?;
                    }
                    if mode == TimingMode::SerialLatency && operation + 1 < iterations {
                        self.record_dependency(command);
                    }
                }
                self.device.cmd_write_timestamp(
                    command,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    query,
                    1,
                );
                self.device.end_command_buffer(command)?;
            }
            lanes.push(Lane {
                command,
                query,
                fence,
            });
        }
        Ok(lanes)
    }

    unsafe fn record_dispatch(
        &self,
        command: vk::CommandBuffer,
        kernel: &str,
        groups: u32,
        spec: &RowSpec,
        offset: u32,
        second: bool,
    ) -> Result<()> {
        let block = if kernel == "memory_interleave4"
            && matches!(
                spec.kernel,
                "memory_interleave4_block64" | "memory_interleave4_block64_b32"
            ) {
            256
        } else {
            spec.stage_block(second)
        };
        let pipeline = *self
            .pipelines
            .get(&(kernel, block))
            .with_context(|| format!("missing Vulkan pipeline {kernel} wg{block}"))?;
        self.device
            .cmd_bind_pipeline(command, vk::PipelineBindPoint::COMPUTE, pipeline);
        let push = [
            spec.n0,
            spec.stage_n1(second),
            offset,
            spec.stage_aux(second),
        ];
        self.device.cmd_push_constants(
            command,
            self.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            words_as_bytes(&push),
        );
        self.device.cmd_dispatch(command, groups, 1, 1);
        Ok(())
    }

    unsafe fn record_dependency(&self, command: vk::CommandBuffer) {
        let barrier = [vk::MemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)];
        self.device.cmd_pipeline_barrier(
            command,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::DependencyFlags::empty(),
            &barrier,
            &[],
            &[],
        );
    }

    fn run_once(&self, lanes: &[Lane], out: &GpuBuffer, iterations: usize) -> Result<f64> {
        self.zero(out)?;
        let fences = lanes.iter().map(|l| l.fence).collect::<Vec<_>>();
        unsafe {
            self.device.reset_fences(&fences)?;
        }
        for (i, lane) in lanes.iter().enumerate() {
            let commands = [lane.command];
            let submit = [vk::SubmitInfo::default().command_buffers(&commands)];
            unsafe {
                self.device
                    .queue_submit(self.queues[i], &submit, lane.fence)?;
            }
        }
        unsafe {
            self.device.wait_for_fences(&fences, true, u64::MAX)?;
        }
        let mask = if self.timestamp_valid_bits == 64 {
            u64::MAX
        } else {
            (1u64 << self.timestamp_valid_bits) - 1
        };
        let mut first = u64::MAX;
        let mut last = 0u64;
        for lane in lanes {
            let mut values = [0u64; 2];
            unsafe {
                self.device.get_query_pool_results(
                    lane.query,
                    0,
                    &mut values,
                    vk::QueryResultFlags::TYPE_64 | vk::QueryResultFlags::WAIT,
                )?;
            }
            first = first.min(values[0] & mask);
            last = last.max(values[1] & mask);
        }
        let ticks = last.wrapping_sub(first) & mask;
        Ok(ticks as f64 * self.timestamp_period_ns / 1000.0 / iterations as f64)
    }

    fn create_descriptor(
        &self,
        a: &GpuBuffer,
        b: &GpuBuffer,
        out: &GpuBuffer,
    ) -> Result<Descriptor> {
        let pool_size = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(3)];
        let pool = unsafe {
            self.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&pool_size),
                None,
            )
        }?;
        let layouts = [self.descriptor_layout];
        let set = unsafe {
            self.device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pool)
                    .set_layouts(&layouts),
            )?
        }[0];
        let infos = [
            vk::DescriptorBufferInfo::default()
                .buffer(a.buffer)
                .offset(0)
                .range(a.size),
            vk::DescriptorBufferInfo::default()
                .buffer(b.buffer)
                .offset(0)
                .range(b.size),
            vk::DescriptorBufferInfo::default()
                .buffer(out.buffer)
                .offset(0)
                .range(out.size),
        ];
        let writes = [0u32, 1, 2].map(|binding| {
            vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(binding)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&infos[binding as usize]))
        });
        unsafe { self.device.update_descriptor_sets(&writes, &[]) };
        Ok(Descriptor { pool, set })
    }

    fn create_device_buffer(&self, size: vk::DeviceSize) -> Result<GpuBuffer> {
        self.create_buffer(
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER
                | vk::BufferUsageFlags::TRANSFER_SRC
                | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
    }

    fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
    ) -> Result<GpuBuffer> {
        let buffer = unsafe {
            self.device.create_buffer(
                &vk::BufferCreateInfo::default()
                    .size(size.max(4))
                    .usage(usage)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE),
                None,
            )
        }?;
        let req = unsafe { self.device.get_buffer_memory_requirements(buffer) };
        let memory_type = self.memory_type(req.memory_type_bits, memory_flags)?;
        let memory = unsafe {
            self.device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(req.size)
                    .memory_type_index(memory_type),
                None,
            )
        }?;
        unsafe {
            self.device.bind_buffer_memory(buffer, memory, 0)?;
        }
        Ok(GpuBuffer {
            buffer,
            memory,
            size: size.max(4),
        })
    }

    fn memory_type(&self, bits: u32, flags: vk::MemoryPropertyFlags) -> Result<u32> {
        for i in 0..self.memory_properties.memory_type_count {
            let ty = self.memory_properties.memory_types[i as usize];
            if bits & (1 << i) != 0 && ty.property_flags.contains(flags) {
                return Ok(i);
            }
        }
        bail!("no Vulkan memory type for {flags:?}")
    }

    fn upload(&self, dst: &GpuBuffer, bytes: &[u8]) -> Result<()> {
        let staging = self.create_buffer(
            dst.size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        unsafe {
            let mapped =
                self.device
                    .map_memory(staging.memory, 0, dst.size, vk::MemoryMapFlags::empty())?;
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.cast::<u8>(), bytes.len());
            self.device.unmap_memory(staging.memory);
        }
        self.one_time(|cmd| unsafe {
            self.device.cmd_copy_buffer(
                cmd,
                staging.buffer,
                dst.buffer,
                &[vk::BufferCopy::default().size(bytes.len() as u64)],
            );
            let barrier = [vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)];
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &barrier,
                &[],
                &[],
            );
        })?;
        unsafe {
            self.destroy_buffer(staging);
        }
        Ok(())
    }

    fn zero(&self, out: &GpuBuffer) -> Result<()> {
        self.one_time(|cmd| unsafe {
            self.device
                .cmd_fill_buffer(cmd, out.buffer, 0, vk::WHOLE_SIZE, 0);
            let barrier = [vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)];
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &barrier,
                &[],
                &[],
            );
        })
    }

    fn download_words(&self, src: &GpuBuffer, words: usize) -> Result<Vec<u32>> {
        unsafe {
            self.device.device_wait_idle()?;
        }
        let bytes_len = words * 4;
        let staging = self.create_buffer(
            src.size,
            vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        self.one_time(|cmd| unsafe {
            let barrier = [vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)];
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &barrier,
                &[],
                &[],
            );
            self.device.cmd_copy_buffer(
                cmd,
                src.buffer,
                staging.buffer,
                &[vk::BufferCopy::default().size(bytes_len as u64)],
            );
        })?;
        let mut bytes = vec![0u8; bytes_len];
        unsafe {
            let mapped = self.device.map_memory(
                staging.memory,
                0,
                bytes_len as u64,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(mapped.cast::<u8>(), bytes.as_mut_ptr(), bytes_len);
            self.device.unmap_memory(staging.memory);
            self.destroy_buffer(staging);
        }
        Ok(bytes
            .chunks_exact(4)
            .map(|v| u32::from_ne_bytes(v.try_into().unwrap()))
            .collect())
    }

    fn one_time(&self, record: impl FnOnce(vk::CommandBuffer)) -> Result<()> {
        let command = unsafe {
            self.device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )
        }?[0];
        unsafe {
            self.device.begin_command_buffer(
                command,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }
        record(command);
        unsafe {
            self.device.end_command_buffer(command)?;
            let commands = [command];
            self.device.queue_submit(
                self.queues[0],
                &[vk::SubmitInfo::default().command_buffers(&commands)],
                vk::Fence::null(),
            )?;
            self.device.queue_wait_idle(self.queues[0])?;
            self.device
                .free_command_buffers(self.command_pool, &[command]);
        }
        Ok(())
    }

    fn destroy_lanes(&self, lanes: Vec<Lane>) {
        unsafe {
            for lane in &lanes {
                self.device.destroy_fence(lane.fence, None);
                self.device.destroy_query_pool(lane.query, None);
            }
            self.device.free_command_buffers(
                self.command_pool,
                &lanes.iter().map(|l| l.command).collect::<Vec<_>>(),
            );
        }
    }

    unsafe fn destroy_buffer(&self, buffer: GpuBuffer) {
        self.device.destroy_buffer(buffer.buffer, None);
        self.device.free_memory(buffer.memory, None);
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            for pipeline in self.pipelines.values() {
                self.device.destroy_pipeline(*pipeline, None);
            }
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_layout, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn words_as_bytes(words: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), words.len() * 4) }
}
