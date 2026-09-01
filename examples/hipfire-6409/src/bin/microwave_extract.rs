// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
use anyhow::{bail, Context, Result};
use ash::{vk, Entry};
use std::ffi::{CStr, CString};
use std::fs;
use std::path::PathBuf;

fn normalize_pci(v: &str) -> String { v.trim().to_ascii_lowercase() }
fn physical_pci(instance: &ash::Instance, physical: vk::PhysicalDevice) -> Option<String> {
    let mut pci_info = vk::PhysicalDevicePCIBusInfoPropertiesEXT::default();
    let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut pci_info);
    unsafe { instance.get_physical_device_properties2(physical, &mut props2) };
    (pci_info.pci_domain != 0 || pci_info.pci_bus != 0 || pci_info.pci_device != 0).then(|| format!("{:04x}:{:02x}:{:02x}.{}", pci_info.pci_domain, pci_info.pci_bus, pci_info.pci_device, pci_info.pci_function))
}
fn parse_args() -> Result<(PathBuf, Option<String>, Option<String>, u32, PathBuf)> {
    let args: Vec<String> = std::env::args().collect();
    let mut spv: Option<PathBuf> = None;
    let mut device_name: Option<String> = None;
    let mut pci: Option<String> = None;
    let mut subgroup: Option<u32> = None;
    let mut out_prefix: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--spv" => { i+=1; if i>=args.len(){bail!("--spv requires path");} spv=Some(PathBuf::from(&args[i])); }
            "--device-name" => { i+=1; if i>=args.len(){bail!("--device-name requires substring");} device_name=Some(args[i].clone()); }
            "--pci" => { i+=1; if i>=args.len(){bail!("--pci requires BDF");} pci=Some(args[i].clone()); }
            "--subgroup" => { i+=1; if i>=args.len(){bail!("--subgroup requires 32|64");} subgroup=Some(args[i].parse().context("subgroup must be 32 or 64")?); }
            "--out-prefix" => { i+=1; if i>=args.len(){bail!("--out-prefix requires path");} out_prefix=Some(PathBuf::from(&args[i])); }
            _ => bail!("unknown arg {}", args[i]),
        }
        i+=1;
    }
    let spv=spv.context("--spv required")?;
    let subgroup=subgroup.context("--subgroup required")?;
    if subgroup!=32 && subgroup!=64 {bail!("subgroup must be 32 or 64");}
    let out_prefix=out_prefix.context("--out-prefix required")?;
    if device_name.is_none()&&pci.is_none(){bail!("need --device-name or --pci");}
    if device_name.is_some()&&pci.is_some(){bail!("only one of --device-name or --pci");}
    Ok((spv, device_name, pci, subgroup, out_prefix))
}
fn main() -> Result<()> {
    let (spv_path, device_name, pci, subgroup, out_prefix) = parse_args()?;
    let spv_bytes = fs::read(&spv_path).with_context(|| format!("read {}", spv_path.display()))?;
    let file_name = out_prefix.file_name().and_then(|s| s.to_str()).unwrap_or("kernel");
    let kernel = if let Some(pos)=file_name.rfind("_w"){ file_name[..pos].to_string()} else {file_name.to_string()};
    let entry = unsafe { Entry::load() }.context("load vulkan loader")?;
    let app_name = CString::new("microwave-extract")?;
    let app = vk::ApplicationInfo::default().application_name(&app_name).application_version(1).engine_name(&app_name).engine_version(1).api_version(vk::API_VERSION_1_3);
    let instance = unsafe { entry.create_instance(&vk::InstanceCreateInfo::default().application_info(&app), None) }.context("create instance")?;
    let physicals = unsafe { instance.enumerate_physical_devices() }.context("enumerate")?;
    let expected_pci = pci.as_deref().map(normalize_pci);
    let expected_name = device_name.as_deref().map(|s| s.to_ascii_lowercase());
    let mut chosen: Option<vk::PhysicalDevice>=None;
    let mut chosen_props: Option<vk::PhysicalDeviceProperties>=None;
    for &phys in &physicals {
        let props = unsafe { instance.get_physical_device_properties(phys) };
        let name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) }.to_string_lossy().to_ascii_lowercase();
        if props.vendor_id!=0x1002{continue;}
        if let Some(ref need)=expected_name { if !name.contains(need){continue;}}
        if let Some(ref need_pci)=expected_pci { if let Some(actual)=physical_pci(&instance, phys){ if normalize_pci(&actual)!=*need_pci{continue;}} else {continue;}}
        chosen=Some(phys); chosen_props=Some(props); break;
    }
    let physical=chosen.context("no matching AMD physical device")?;
    let props=chosen_props.unwrap();
    let device_name_str=unsafe{CStr::from_ptr(props.device_name.as_ptr())}.to_string_lossy().into_owned();
    let mut driver_props=vk::PhysicalDeviceDriverProperties::default();
    let mut props2=vk::PhysicalDeviceProperties2::default().push_next(&mut driver_props);
    unsafe{instance.get_physical_device_properties2(physical, &mut props2)};
    let driver_info=unsafe{CStr::from_ptr(driver_props.driver_info.as_ptr())}.to_string_lossy().into_owned();
    let mesa=driver_info.clone();
    let device_str=device_name_str.clone();
    let arch={
        let lower=device_str.to_ascii_lowercase();
        if lower.contains("gfx1201") || lower.contains("navi48") || lower.contains("9070") {"gfx1201".to_string()}
        else if lower.contains("gfx1151") || lower.contains("strix_halo") || lower.contains("8060s") || lower.contains("8050s") {"gfx1151".to_string()}
        else if lower.contains("gfx1100") || lower.contains("navi31") || lower.contains("7900 xtx") {"gfx1100".to_string()}
        else if lower.contains("gfx1030") || lower.contains("navi21") {"gfx1030".to_string()}
        else {"unknown".to_string()}
    };
    let exts=unsafe{instance.enumerate_device_extension_properties(physical)}.context("enumerate device ext")?;
    let ext_names: std::collections::HashSet<String>=exts.iter().map(|e| unsafe{CStr::from_ptr(e.extension_name.as_ptr()).to_string_lossy().into_owned()}).collect();
    let has_exec=ext_names.contains("VK_KHR_pipeline_executable_properties");
    let has_subgroup=ext_names.contains("VK_EXT_subgroup_size_control");
    let families=unsafe{instance.get_physical_device_queue_family_properties(physical)};
    let (qfi,_)=families.iter().enumerate().find(|(_,f)| f.queue_flags.contains(vk::QueueFlags::COMPUTE)).context("no compute queue")?;
    let qfi=qfi as u32;
    let mut dot_features=vk::PhysicalDeviceShaderIntegerDotProductFeatures::default().shader_integer_dot_product(true);
    let mut bda_features=vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);
    let mut subgroup_features=vk::PhysicalDeviceSubgroupSizeControlFeatures::default().subgroup_size_control(true).compute_full_subgroups(true);
    let mut exec_features=vk::PhysicalDevicePipelineExecutablePropertiesFeaturesKHR::default().pipeline_executable_info(true);
    let mut features2=vk::PhysicalDeviceFeatures2::default();
    features2=features2.push_next(&mut exec_features);
    features2=features2.push_next(&mut subgroup_features);
    features2=features2.push_next(&mut bda_features);
    features2=features2.push_next(&mut dot_features);
    let queue_priority=[1.0f32];
    let queue_info=vk::DeviceQueueCreateInfo::default().queue_family_index(qfi).queue_priorities(&queue_priority);
    let mut ext_list: Vec<CString>=Vec::new();
    if has_exec{ext_list.push(CString::new("VK_KHR_pipeline_executable_properties").unwrap());}
    if has_subgroup{ext_list.push(CString::new("VK_EXT_subgroup_size_control").unwrap());}
    if ext_names.contains("VK_KHR_buffer_device_address"){ext_list.push(CString::new("VK_KHR_buffer_device_address").unwrap());}
    let ext_ptrs: Vec<*const i8>=ext_list.iter().map(|s| s.as_ptr()).collect();
    let device_info=vk::DeviceCreateInfo::default().queue_create_infos(std::slice::from_ref(&queue_info)).enabled_extension_names(&ext_ptrs).push_next(&mut features2);
    let device=unsafe{instance.create_device(physical, &device_info, None)}.context("create device")?;
    let spv_words=ash::util::read_spv(&mut std::io::Cursor::new(&spv_bytes)).context("read spv")?;
    let shader_info=vk::ShaderModuleCreateInfo::default().code(&spv_words);
    let shader_module=unsafe{device.create_shader_module(&shader_info, None)}.context("create shader module")?;
    let push_range=vk::PushConstantRange::default().stage_flags(vk::ShaderStageFlags::COMPUTE).offset(0).size(12);
    let pipeline_layout=unsafe{device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default().push_constant_ranges(std::slice::from_ref(&push_range)), None)}.context("pipeline layout")?;
    let block: u32=64;
    let map_entries=[vk::SpecializationMapEntry{constant_id:0, offset:0, size:4}];
    let spec_data=block.to_ne_bytes();
    let spec_info=vk::SpecializationInfo::default().map_entries(&map_entries).data(&spec_data);
    let mut subgroup_info=vk::PipelineShaderStageRequiredSubgroupSizeCreateInfo::default().required_subgroup_size(subgroup);
    let stage=vk::PipelineShaderStageCreateInfo::default().stage(vk::ShaderStageFlags::COMPUTE).module(shader_module).name(CStr::from_bytes_with_nul(b"main\0").unwrap()).specialization_info(&spec_info).push_next(&mut subgroup_info);
    let pipeline_info=vk::ComputePipelineCreateInfo::default().stage(stage).layout(pipeline_layout).flags(vk::PipelineCreateFlags::CAPTURE_STATISTICS_KHR|vk::PipelineCreateFlags::CAPTURE_INTERNAL_REPRESENTATIONS_KHR);
    let pipelines=unsafe{device.create_compute_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None)}.map_err(|e| anyhow::anyhow!("create pipeline: {:?}", e.1))?;
    let pipeline=pipelines[0];
    let exec_device=ash::khr::pipeline_executable_properties::Device::new(&instance, &device);
    let props_vec=unsafe{exec_device.get_pipeline_executable_properties(&vk::PipelineInfoKHR::default().pipeline(pipeline))}.context("get exec props")?;
    if props_vec.is_empty(){bail!("no executable properties");}
    let exec_index=0u32;
    let stats=unsafe{exec_device.get_pipeline_executable_statistics(&vk::PipelineExecutableInfoKHR::default().pipeline(pipeline).executable_index(exec_index))}.context("get stats")?;
    // Manual IR retrieval
    let executable_info=vk::PipelineExecutableInfoKHR::default().pipeline(pipeline).executable_index(exec_index);
    let fp: vk::PFN_vkGetPipelineExecutableInternalRepresentationsKHR = unsafe {
        let name=CStr::from_bytes_with_nul(b"vkGetPipelineExecutableInternalRepresentationsKHR\0").unwrap();
        let ptr=instance.get_device_proc_addr(device.handle(), name.as_ptr());
        if ptr.is_none(){bail!("missing vkGetPipelineExecutableInternalRepresentationsKHR");}
        std::mem::transmute(ptr.unwrap())
    };
    let mut ir_count: u32=0;
    unsafe{fp(device.handle(), &executable_info, &mut ir_count, std::ptr::null_mut());}
    if ir_count==0{bail!("no internal representations");}
    let mut irs=vec![vk::PipelineExecutableInternalRepresentationKHR::default(); ir_count as usize];
    unsafe{fp(device.handle(), &executable_info, &mut ir_count, irs.as_mut_ptr());}
    let mut data_buffers: Vec<Vec<u8>>=Vec::with_capacity(irs.len());
    for ir in &mut irs{ let size=ir.data_size; let mut buf=vec![0u8; size]; ir.p_data=buf.as_mut_ptr() as *mut std::ffi::c_void; data_buffers.push(buf);}
    unsafe{fp(device.handle(), &executable_info, &mut ir_count, irs.as_mut_ptr());}
    for s in &stats{let name=unsafe{CStr::from_ptr(s.name.as_ptr()).to_string_lossy()}; eprintln!("stat {} format {:?}", name, s.format);}
    for ir in &irs{let name=unsafe{CStr::from_ptr(ir.name.as_ptr()).to_string_lossy()}; let desc=unsafe{CStr::from_ptr(ir.description.as_ptr()).to_string_lossy()}; eprintln!("IR name '{}' desc '{}' is_text {} size {}", name, desc, ir.is_text, ir.data_size);}
    let mut vgprs_u=0u32; let mut sgprs_u=0u32; let mut spilled_vgprs_u=0u32; let mut spilled_sgprs_u=0u32; let mut lds_bytes=0u32; let mut scratch_bytes=0u32; let mut code_size=0u32;
    for st in &stats{
        let name=unsafe{CStr::from_ptr(st.name.as_ptr()).to_string_lossy().to_ascii_lowercase()};
        let val=match st.format{ vk::PipelineExecutableStatisticFormatKHR::UINT64=>unsafe{st.value.u64 as i64}, vk::PipelineExecutableStatisticFormatKHR::INT64=>unsafe{st.value.i64}, vk::PipelineExecutableStatisticFormatKHR::BOOL32=>unsafe{st.value.b32 as i64}, _=>0,};
        if name.contains("vgpr") && !name.contains("spill"){vgprs_u=val as u32;}
        if name.contains("spilled") && name.contains("vgpr"){spilled_vgprs_u=val as u32;}
        if name.contains("sgpr") && !name.contains("spill") && sgprs_u==0 || name=="sgprs"{ if name=="sgprs" || sgprs_u==0{sgprs_u=val as u32;}}
        if name.contains("spilled") && name.contains("sgpr"){spilled_sgprs_u=val as u32;}
        if name.contains("lds"){lds_bytes=val as u32;}
        if name.contains("scratch"){scratch_bytes=val as u32;}
        if name.contains("code size") || name=="code_size"{code_size=val as u32;}
    }
    let mut asm_text: Option<String>=None; let mut aco_text: Option<String>=None; let mut nir_text: Option<String>=None;
    for ir in &irs{
        let name=unsafe{CStr::from_ptr(ir.name.as_ptr()).to_string_lossy().to_ascii_lowercase()};
        let desc=unsafe{CStr::from_ptr(ir.description.as_ptr()).to_string_lossy().to_ascii_lowercase()};
        let data_slice=unsafe{std::slice::from_raw_parts(ir.p_data as *const u8, ir.data_size)};
        let text=String::from_utf8_lossy(data_slice).into_owned();
        if name.contains("assembly") || desc.contains("assembly") || (name.contains("amd") && text.contains("s_endpgm")){asm_text=Some(text);} else if name.contains("aco") || desc.contains("aco"){aco_text=Some(text);} else if name.contains("nir") || desc.contains("nir"){nir_text=Some(text);}
    }
    if asm_text.is_none() || aco_text.is_none() || nir_text.is_none(){
        for ir in &irs{
            let data_slice=unsafe{std::slice::from_raw_parts(ir.p_data as *const u8, ir.data_size)};
            let text=String::from_utf8_lossy(data_slice).into_owned();
            if text.contains("p_startpgm") && aco_text.is_none(){aco_text=Some(text.clone());} else if text.contains("load_push_constant") && nir_text.is_none(){nir_text=Some(text.clone());} else if text.contains("s_endpgm") && asm_text.is_none(){asm_text=Some(text.clone());}
        }
    }
    let asm=asm_text.context("missing Assembly IR")?;
    let aco=aco_text.unwrap_or_default();
    let nir=nir_text.unwrap_or_default();
    // ABI per report
    let (tgid_x, args_ptr, wg_size) = if arch=="gfx1201" {
        ("ttmp9".to_string(), "s[2:3]".to_string(), "s4".to_string())
    } else if arch=="gfx1151" || arch=="gfx1100" {
        ("s[5]".to_string(), "s[2:3]".to_string(), "s4".to_string())
    } else {
        if asm.contains("ttmp9"){("ttmp9".to_string(),"s[2:3]".to_string(),"s4".to_string())} else {("s[5]".to_string(),"s[2:3]".to_string(),"s4".to_string())}
    };
    // validate
    if asm.contains("ttmp9") && tgid_x!="ttmp9"{ eprintln!("warning: asm contains ttmp9 but arch {} expected {}", arch, tgid_x); }
    for line in aco.lines(){ if line.contains("load_push_constant"){eprintln!("ACO load: {}", line);}}
    for line in nir.lines(){ if line.contains("load_push_constant"){eprintln!("NIR load: {}", line);}}
    let code_size_out=if code_size==0{asm.len() as u32}else{code_size};
    if let Some(parent)=out_prefix.parent(){fs::create_dir_all(parent).context("create out dir")?;}
    let s_path=out_prefix.with_extension("s");
    let aco_path=out_prefix.with_extension("aco");
    let nir_path=out_prefix.with_extension("nir");
    let json_path=out_prefix.with_extension("json");
    fs::write(&s_path, &asm).context("write s")?;
    fs::write(&aco_path, &aco).context("write aco")?;
    fs::write(&nir_path, &nir).context("write nir")?;
    let json=serde_json::json!({"arch":arch,"kernel":kernel,"wave":subgroup,"vgprs":vgprs_u,"sgprs":sgprs_u,"spilled_vgprs":spilled_vgprs_u,"spilled_sgprs":spilled_sgprs_u,"lds_bytes":lds_bytes,"scratch_bytes":scratch_bytes,"code_size":code_size_out,"abi":{"args_ptr":args_ptr,"wg_size":wg_size,"tgid_x":tgid_x,"tid":"v0"},"mesa":mesa,"device":device_str});
    fs::write(&json_path, serde_json::to_string_pretty(&json)?).context("write json")?;
    println!("wrote {} vgprs={} sgprs={} lds={} scratch={} code={}", json_path.display(), vgprs_u, sgprs_u, lds_bytes, scratch_bytes, code_size_out);
    println!("abi args_ptr={} wg_size={} tgid_x={} tid=v0", args_ptr, wg_size, tgid_x);
    unsafe{device.destroy_pipeline(pipeline, None); device.destroy_pipeline_layout(pipeline_layout, None); device.destroy_shader_module(shader_module, None); device.destroy_device(None); instance.destroy_instance(None);}
    Ok(())
}
