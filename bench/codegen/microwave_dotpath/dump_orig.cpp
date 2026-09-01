// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Dump RADV/ACO assembly for a compute SPIR-V. Supports the microwave
// pointer-ABI layout (3-dword push const, no descriptor sets) and the
// original hipEngine shader (20-byte push const, one set of 3 storage
// buffers). Also prints timestampPeriod.
//
//   c++ -O2 -std=c++17 dump_orig.cpp -lvulkan -o dump_orig
//   ./dump_orig --spv foo.spv --device-name GFX1151 --subgroup 64 \
//       --push-size 20 --storage-buffers 3 --out-prefix orig_w64

#include <vulkan/vulkan.h>
#include <cctype>
#include <cstdlib>

#include <cstdint>
#include <cstring>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

static void die(const std::string& m) {
  std::cerr << m << "\n";
  std::exit(1);
}
static void ck(VkResult r, const char* w) {
  if (r != VK_SUCCESS) {
    die(std::string(w) + " failed: " + std::to_string(r));
  }
}

int main(int argc, char** argv) {
  std::string spv, device_name, out_prefix;
  uint32_t subgroup = 0, push_size = 12, storage_buffers = 0;
  for (int i = 1; i < argc; ++i) {
    auto need = [&](const char* f) {
      if (i + 1 >= argc) die(std::string(f) + " requires value");
      return std::string(argv[++i]);
    };
    std::string a = argv[i];
    if (a == "--spv")
      spv = need("--spv");
    else if (a == "--device-name")
      device_name = need("--device-name");
    else if (a == "--subgroup")
      subgroup = static_cast<uint32_t>(std::stoul(need("--subgroup")));
    else if (a == "--out-prefix")
      out_prefix = need("--out-prefix");
    else if (a == "--push-size")
      push_size = static_cast<uint32_t>(std::stoul(need("--push-size")));
    else if (a == "--storage-buffers")
      storage_buffers = static_cast<uint32_t>(std::stoul(need("--storage-buffers")));
    else
      die("unknown arg " + a);
  }
  if (spv.empty() || device_name.empty() || out_prefix.empty() || (subgroup != 32 && subgroup != 64))
    die("need --spv --device-name --subgroup 32|64 --out-prefix");

  std::ifstream in(spv, std::ios::binary);
  if (!in) die("read spv");
  std::vector<char> bytes((std::istreambuf_iterator<char>(in)), {});
  if (bytes.size() < 4 || bytes.size() % 4) die("bad spv size");
  std::vector<uint32_t> words(bytes.size() / 4);
  std::memcpy(words.data(), bytes.data(), bytes.size());

  VkApplicationInfo app{VK_STRUCTURE_TYPE_APPLICATION_INFO};
  app.pApplicationName = "dump-orig";
  app.apiVersion = VK_API_VERSION_1_3;
  VkInstanceCreateInfo ici{VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO};
  ici.pApplicationInfo = &app;
  VkInstance instance;
  ck(vkCreateInstance(&ici, nullptr, &instance), "vkCreateInstance");

  uint32_t nphys = 0;
  ck(vkEnumeratePhysicalDevices(instance, &nphys, nullptr), "enum phys");
  std::vector<VkPhysicalDevice> phys(nphys);
  ck(vkEnumeratePhysicalDevices(instance, &nphys, phys.data()), "enum phys 2");
  VkPhysicalDevice chosen = VK_NULL_HANDLE;
  VkPhysicalDeviceProperties props{};
  std::string want = device_name;
  for (char& c : want) c = static_cast<char>(std::tolower(c));
  for (auto p : phys) {
    vkGetPhysicalDeviceProperties(p, &props);
    if (props.vendorID != 0x1002) continue;
    std::string name = props.deviceName;
    std::string lower = name;
    for (char& c : lower) c = static_cast<char>(std::tolower(c));
    if (lower.find(want) == std::string::npos) continue;
    chosen = p;
    break;
  }
  if (!chosen) die("no matching AMD device for " + device_name);
  vkGetPhysicalDeviceProperties(chosen, &props);
  std::cout << "device=" << props.deviceName << "\n";
  std::cout << "timestampPeriod=" << props.limits.timestampPeriod << "\n";

  VkPhysicalDeviceDriverProperties drv{VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DRIVER_PROPERTIES};
  VkPhysicalDeviceProperties2 p2{VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2};
  p2.pNext = &drv;
  vkGetPhysicalDeviceProperties2(chosen, &p2);
  std::cout << "driverInfo=" << drv.driverInfo << "\n";

  uint32_t nfam = 0;
  vkGetPhysicalDeviceQueueFamilyProperties(chosen, &nfam, nullptr);
  std::vector<VkQueueFamilyProperties> fams(nfam);
  vkGetPhysicalDeviceQueueFamilyProperties(chosen, &nfam, fams.data());
  uint32_t qfi = 0;
  bool found = false;
  for (uint32_t i = 0; i < nfam; ++i) {
    if (fams[i].queueFlags & VK_QUEUE_COMPUTE_BIT) {
      qfi = i;
      found = true;
      break;
    }
  }
  if (!found) die("no compute queue");

  uint32_t nexts = 0;
  vkEnumerateDeviceExtensionProperties(chosen, nullptr, &nexts, nullptr);
  std::vector<VkExtensionProperties> exts(nexts);
  vkEnumerateDeviceExtensionProperties(chosen, nullptr, &nexts, exts.data());
  auto has = [&](const char* n) {
    for (auto& e : exts)
      if (std::strcmp(e.extensionName, n) == 0) return true;
    return false;
  };
  std::vector<const char*> en;
  if (has(VK_KHR_PIPELINE_EXECUTABLE_PROPERTIES_EXTENSION_NAME))
    en.push_back(VK_KHR_PIPELINE_EXECUTABLE_PROPERTIES_EXTENSION_NAME);
  if (has(VK_EXT_SUBGROUP_SIZE_CONTROL_EXTENSION_NAME))
    en.push_back(VK_EXT_SUBGROUP_SIZE_CONTROL_EXTENSION_NAME);
  if (has("VK_KHR_shader_integer_dot_product"))
    en.push_back("VK_KHR_shader_integer_dot_product");

  VkPhysicalDeviceShaderIntegerDotProductFeatures dotf{
      VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SHADER_INTEGER_DOT_PRODUCT_FEATURES};
  dotf.shaderIntegerDotProduct = VK_TRUE;
  VkPhysicalDeviceSubgroupSizeControlFeatures subf{
      VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SUBGROUP_SIZE_CONTROL_FEATURES};
  subf.subgroupSizeControl = VK_TRUE;
  subf.computeFullSubgroups = VK_TRUE;
  subf.pNext = &dotf;
  VkPhysicalDevicePipelineExecutablePropertiesFeaturesKHR execf{
      VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PIPELINE_EXECUTABLE_PROPERTIES_FEATURES_KHR};
  execf.pipelineExecutableInfo = VK_TRUE;
  execf.pNext = &subf;
  VkPhysicalDeviceFeatures2 feat{VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2};
  feat.pNext = &execf;

  float prio = 1.f;
  VkDeviceQueueCreateInfo qci{VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO};
  qci.queueFamilyIndex = qfi;
  qci.queueCount = 1;
  qci.pQueuePriorities = &prio;
  VkDeviceCreateInfo dci{VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO};
  dci.pNext = &feat;
  dci.queueCreateInfoCount = 1;
  dci.pQueueCreateInfos = &qci;
  dci.enabledExtensionCount = static_cast<uint32_t>(en.size());
  dci.ppEnabledExtensionNames = en.data();
  VkDevice device;
  ck(vkCreateDevice(chosen, &dci, nullptr, &device), "vkCreateDevice");

  VkShaderModuleCreateInfo smci{VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO};
  smci.codeSize = bytes.size();
  smci.pCode = words.data();
  VkShaderModule sm;
  ck(vkCreateShaderModule(device, &smci, nullptr, &sm), "shader module");

  VkDescriptorSetLayout dsl = VK_NULL_HANDLE;
  VkDescriptorSetLayoutCreateInfo dslci{VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO};
  std::vector<VkDescriptorSetLayoutBinding> binds;
  if (storage_buffers) {
    binds.resize(storage_buffers);
    for (uint32_t i = 0; i < storage_buffers; ++i) {
      binds[i].binding = i;
      binds[i].descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
      binds[i].descriptorCount = 1;
      binds[i].stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
    }
    dslci.bindingCount = storage_buffers;
    dslci.pBindings = binds.data();
    ck(vkCreateDescriptorSetLayout(device, &dslci, nullptr, &dsl), "dsl");
  }

  VkPushConstantRange pcr{};
  pcr.stageFlags = VK_SHADER_STAGE_COMPUTE_BIT;
  pcr.offset = 0;
  pcr.size = push_size;
  VkPipelineLayoutCreateInfo plci{VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO};
  if (dsl) {
    plci.setLayoutCount = 1;
    plci.pSetLayouts = &dsl;
  }
  if (push_size) {
    plci.pushConstantRangeCount = 1;
    plci.pPushConstantRanges = &pcr;
  }
  VkPipelineLayout layout;
  ck(vkCreatePipelineLayout(device, &plci, nullptr, &layout), "pipeline layout");

  uint32_t block = 64;
  VkSpecializationMapEntry me{0, 0, 4};
  VkSpecializationInfo spec{};
  spec.mapEntryCount = 1;
  spec.pMapEntries = &me;
  spec.dataSize = 4;
  spec.pData = &block;

  VkPipelineShaderStageRequiredSubgroupSizeCreateInfo ssz{
      VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_REQUIRED_SUBGROUP_SIZE_CREATE_INFO};
  ssz.requiredSubgroupSize = subgroup;
  VkPipelineShaderStageCreateInfo stage{VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO};
  stage.pNext = &ssz;
  stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
  stage.module = sm;
  stage.pName = "main";
  stage.pSpecializationInfo = &spec;

  VkComputePipelineCreateInfo pci{VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO};
  pci.flags = VK_PIPELINE_CREATE_CAPTURE_STATISTICS_BIT_KHR |
              VK_PIPELINE_CREATE_CAPTURE_INTERNAL_REPRESENTATIONS_BIT_KHR;
  pci.stage = stage;
  pci.layout = layout;
  VkPipeline pipeline;
  ck(vkCreateComputePipelines(device, VK_NULL_HANDLE, 1, &pci, nullptr, &pipeline), "pipeline");

  auto get_exec_props = reinterpret_cast<PFN_vkGetPipelineExecutablePropertiesKHR>(
      vkGetDeviceProcAddr(device, "vkGetPipelineExecutablePropertiesKHR"));
  auto get_stats = reinterpret_cast<PFN_vkGetPipelineExecutableStatisticsKHR>(
      vkGetDeviceProcAddr(device, "vkGetPipelineExecutableStatisticsKHR"));
  auto get_ir = reinterpret_cast<PFN_vkGetPipelineExecutableInternalRepresentationsKHR>(
      vkGetDeviceProcAddr(device, "vkGetPipelineExecutableInternalRepresentationsKHR"));
  if (!get_exec_props || !get_stats || !get_ir) die("missing executable-properties entry points");

  VkPipelineInfoKHR pi{VK_STRUCTURE_TYPE_PIPELINE_INFO_KHR};
  pi.pipeline = pipeline;
  uint32_t nexec = 0;
  ck(get_exec_props(device, &pi, &nexec, nullptr), "exec props count");
  std::vector<VkPipelineExecutablePropertiesKHR> eprops(nexec);
  for (auto& e : eprops) e.sType = VK_STRUCTURE_TYPE_PIPELINE_EXECUTABLE_PROPERTIES_KHR;
  ck(get_exec_props(device, &pi, &nexec, eprops.data()), "exec props");
  VkPipelineExecutableInfoKHR ei{VK_STRUCTURE_TYPE_PIPELINE_EXECUTABLE_INFO_KHR};
  ei.pipeline = pipeline;
  ei.executableIndex = 0;

  uint32_t nstat = 0;
  ck(get_stats(device, &ei, &nstat, nullptr), "stats count");
  std::vector<VkPipelineExecutableStatisticKHR> stats(nstat);
  for (auto& s : stats) s.sType = VK_STRUCTURE_TYPE_PIPELINE_EXECUTABLE_STATISTIC_KHR;
  ck(get_stats(device, &ei, &nstat, stats.data()), "stats");
  uint32_t vgprs = 0, sgprs = 0, code_size = 0, lds = 0, scratch = 0;
  for (auto& s : stats) {
    std::string n = s.name;
    int64_t val = 0;
    if (s.format == VK_PIPELINE_EXECUTABLE_STATISTIC_FORMAT_UINT64_KHR)
      val = static_cast<int64_t>(s.value.u64);
    else if (s.format == VK_PIPELINE_EXECUTABLE_STATISTIC_FORMAT_INT64_KHR)
      val = s.value.i64;
    std::string low = n;
    for (char& c : low) c = static_cast<char>(std::tolower(c));
    if (low.find("vgpr") != std::string::npos && low.find("spill") == std::string::npos)
      vgprs = static_cast<uint32_t>(val);
    if (low == "sgprs" || (low.find("sgpr") != std::string::npos && low.find("spill") == std::string::npos && !sgprs))
      sgprs = static_cast<uint32_t>(val);
    if (low.find("code size") != std::string::npos) code_size = static_cast<uint32_t>(val);
    if (low.find("lds") != std::string::npos) lds = static_cast<uint32_t>(val);
    if (low.find("scratch") != std::string::npos) scratch = static_cast<uint32_t>(val);
    std::cerr << "stat " << n << "=" << val << "\n";
  }

  uint32_t nir = 0;
  ck(get_ir(device, &ei, &nir, nullptr), "ir count");
  std::vector<VkPipelineExecutableInternalRepresentationKHR> irs(nir);
  for (auto& ir : irs) ir.sType = VK_STRUCTURE_TYPE_PIPELINE_EXECUTABLE_INTERNAL_REPRESENTATION_KHR;
  ck(get_ir(device, &ei, &nir, irs.data()), "ir sizes");
  std::vector<std::vector<char>> bufs(nir);
  for (uint32_t i = 0; i < nir; ++i) {
    bufs[i].assign(irs[i].dataSize, 0);
    irs[i].pData = bufs[i].data();
  }
  ck(get_ir(device, &ei, &nir, irs.data()), "ir data");
  std::string asm_text, aco_text, nir_text;
  for (uint32_t i = 0; i < nir; ++i) {
    std::string n = irs[i].name;
    std::string t(bufs[i].data(), bufs[i].size());
    std::cerr << "IR name '" << n << "' size " << bufs[i].size() << "\n";
    std::string low = n;
    for (char& c : low) c = static_cast<char>(std::tolower(c));
    if (low.find("assembly") != std::string::npos || t.find("s_endpgm") != std::string::npos)
      asm_text = t;
    else if (low.find("aco") != std::string::npos)
      aco_text = t;
    else if (low.find("nir") != std::string::npos)
      nir_text = t;
  }
  if (asm_text.empty()) die("missing Assembly IR");

  auto write = [](const std::string& path, const std::string& s) {
    std::ofstream o(path);
    o << s;
  };
  write(out_prefix + ".s", asm_text);
  write(out_prefix + ".aco", aco_text);
  write(out_prefix + ".nir", nir_text);
  std::ofstream js(out_prefix + ".json");
  js << "{\n  \"device\": \"" << props.deviceName << "\",\n"
     << "  \"mesa\": \"" << drv.driverInfo << "\",\n"
     << "  \"timestampPeriod\": " << props.limits.timestampPeriod << ",\n"
     << "  \"wave\": " << subgroup << ",\n"
     << "  \"vgprs\": " << vgprs << ",\n"
     << "  \"sgprs\": " << sgprs << ",\n"
     << "  \"code_size\": " << code_size << ",\n"
     << "  \"lds_bytes\": " << lds << ",\n"
     << "  \"scratch_bytes\": " << scratch << ",\n"
     << "  \"push_size\": " << push_size << ",\n"
     << "  \"storage_buffers\": " << storage_buffers << "\n}\n";
  std::cout << "wrote " << out_prefix << ".s vgprs=" << vgprs << " sgprs=" << sgprs
            << " code=" << code_size << "\n";

  vkDestroyPipeline(device, pipeline, nullptr);
  vkDestroyPipelineLayout(device, layout, nullptr);
  if (dsl) vkDestroyDescriptorSetLayout(device, dsl, nullptr);
  vkDestroyShaderModule(device, sm, nullptr);
  vkDestroyDevice(device, nullptr);
  vkDestroyInstance(instance, nullptr);
  return 0;
}
