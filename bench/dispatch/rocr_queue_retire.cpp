// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>
//
// Does destroying a real hardware queue retire what its in-flight dispatches
// still reference?
//
// Why this probe exists, and why the earlier one did not answer it:
//   ROCm/ROCm#6529 reports intermittent address-zero SQC-data VM faults on
//   gfx1100 escalating into MES REMOVE_QUEUE failures and MODE1 resets. The
//   leading hypothesis, from the original reporter, is queue/signal/allocation
//   retirement and reuse. That hypothesis has never actually been tested.
//
//   A previous stress used hipStreamCreateWithFlags/hipStreamDestroy, which does
//   NOT create or destroy hardware queues: ROCm 7.14 CLR pools ordinary queues
//   behind GPU_MAX_HW_QUEUES (default 4), so N stream create/destroy pairs are N
//   *stream* operations over a handful of long-lived hardware queues, and
//   stream teardown calls releaseQueue rather than hsa_queue_destroy. A pure-HIP
//   run of 19,200 such pairs with resident kernels was clean on gfx1201 and
//   gfx1100, which is consistent with it never having exercised the path.
//
//   This probe goes straight at ROCr instead: hsa_queue_create and
//   hsa_queue_destroy per cycle, which is a real KFD queue create and destroy.
//
//   Reading ROCr's sources, successful hsa_queue_destroy does not appear to
//   guarantee the hardware can no longer reference the packets' completion
//   signals or kernarg memory: AqlQueue::~AqlQueue waits for the error handler
//   and frees what the queue object owns, Inactivate does an active_ exchange
//   then agent_->driver().DestroyQueue(queue_id_), and KfdDriver::DestroyQueue
//   wraps hsaKmtDestroyQueue -> AMDKFD_IOC_DESTROY_QUEUE plus userspace
//   bookkeeping. There is no completion-signal, kernarg or IB walk on that path.
//   So the question this probe asks is whether an application that frees those
//   objects immediately after a successful destroy can be caught by hardware
//   still reading them.
//
// The shape, per cycle:
//   1. hsa_queue_create                          (real hardware queue)
//   2. allocate a completion signal and kernarg buffer
//   3. enqueue `depth` kernel dispatch packets and ring the doorbell
//   4. hsa_queue_destroy WITHOUT waiting for completion
//   5. immediately destroy the signal and free the kernarg buffer
//   The next cycle reallocates, and the allocator will very likely hand back the
//   same addresses -- which is the reuse half of the hypothesis.
//
// What a result means:
//   Faults land in the kernel log, not in this process, so this probe cannot
//   report them and does not try. It reports what it did. A clean run is weak
//   evidence: #6529 is intermittent, and the upstream sibling report for a
//   related defect notes it reproduced only on specific systems. A fault is
//   strong evidence.
//
// THIS PROBE IS DESIGNED TO STRESS A SUSPECTED USE-AFTER-FREE IN GPU QUEUE
// TEARDOWN. It can fault, wedge, or reset the device. Run it on a headless,
// instrumented, expendable GPU. On this fleet gpusentry is watching and will
// harvest an incident directory; the 2026-07-23 occurrence of this fault
// survived 24 of 24 MODE1 resets on the same card, which bounds the risk but
// does not remove it.
//
// Build:
//   hipcc -O2 rocr_queue_retire.cpp -o rocr_queue_retire \
//       -I/opt/rocm/core-7.14/include -L/opt/rocm/core-7.14/lib -lhsa-runtime64
// Run:
//   ./rocr_queue_retire <kernel.co> [cycles] [depth]

#include <hsa/hsa.h>
#include <hsa/hsa_ext_amd.h>
#include <fcntl.h>
#include <unistd.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#define HSA_OK(x)                                                                \
    do {                                                                         \
        hsa_status_t _s = (x);                                                    \
        if (_s != HSA_STATUS_SUCCESS) {                                           \
            const char* _m = nullptr;                                             \
            hsa_status_string(_s, &_m);                                           \
            printf("FATAL %s -> %d (%s) at line %d\n", #x, (int)_s,               \
                   _m ? _m : "?", __LINE__);                                      \
            return 3;                                                            \
        }                                                                        \
    } while (0)

struct Ctx {
    hsa_agent_t gpu{};
    bool found_gpu = false;
    hsa_agent_t cpu{};
    bool found_cpu = false;
    hsa_amd_memory_pool_t kernarg{};
    bool found_kernarg = false;
};

static hsa_status_t on_agent(hsa_agent_t a, void* data) {
    Ctx* c = (Ctx*)data;
    hsa_device_type_t t;
    if (hsa_agent_get_info(a, HSA_AGENT_INFO_DEVICE, &t) != HSA_STATUS_SUCCESS)
        return HSA_STATUS_SUCCESS;
    if (t == HSA_DEVICE_TYPE_GPU && !c->found_gpu) {
        c->gpu = a;
        c->found_gpu = true;
    }
    // Kernarg memory is host-visible fine-grained system memory, so the pool
    // that carries KERNARG_INIT belongs to the CPU agent, not the GPU.
    if (t == HSA_DEVICE_TYPE_CPU && !c->found_cpu) {
        c->cpu = a;
        c->found_cpu = true;
    }
    return HSA_STATUS_SUCCESS;
}

static hsa_status_t on_pool(hsa_amd_memory_pool_t p, void* data) {
    Ctx* c = (Ctx*)data;
    hsa_amd_segment_t seg;
    if (hsa_amd_memory_pool_get_info(p, HSA_AMD_MEMORY_POOL_INFO_SEGMENT, &seg) !=
        HSA_STATUS_SUCCESS)
        return HSA_STATUS_SUCCESS;
    if (seg != HSA_AMD_SEGMENT_GLOBAL) return HSA_STATUS_SUCCESS;
    uint32_t flags = 0;
    if (hsa_amd_memory_pool_get_info(p, HSA_AMD_MEMORY_POOL_INFO_GLOBAL_FLAGS,
                                     &flags) != HSA_STATUS_SUCCESS)
        return HSA_STATUS_SUCCESS;
    if ((flags & HSA_AMD_MEMORY_POOL_GLOBAL_FLAG_KERNARG_INIT) && !c->found_kernarg) {
        c->kernarg = p;
        c->found_kernarg = true;
    }
    return HSA_STATUS_SUCCESS;
}

// Queues are destroyed while busy on purpose, so ROCr will report asynchronous
// errors on them. Swallow them here: they are the expected consequence of the
// thing under test, not the signal we are looking for (which is in dmesg).
static void queue_err(hsa_status_t st, hsa_queue_t*, void*) {
    static int reported = 0;
    if (reported < 5) {
        const char* m = nullptr;
        hsa_status_string(st, &m);
        printf("  [async queue error %d (%s)]\n", (int)st, m ? m : "?");
        ++reported;
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        printf("usage: %s <kernel.co> [cycles] [depth]\n", argv[0]);
        return 1;
    }
    const char* co_path = argv[1];
    const int cycles = argc > 2 ? atoi(argv[2]) : 2000;
    const int depth = argc > 3 ? atoi(argv[3]) : 16;

    HSA_OK(hsa_init());

    Ctx ctx;
    HSA_OK(hsa_iterate_agents(on_agent, &ctx));
    if (!ctx.found_gpu) { printf("no GPU agent\n"); return 1; }
    if (!ctx.found_cpu) { printf("no CPU agent\n"); return 1; }
    HSA_OK(hsa_amd_agent_iterate_memory_pools(ctx.cpu, on_pool, &ctx));
    if (!ctx.found_kernarg) { printf("no kernarg pool on CPU agent\n"); return 1; }

    char name[64] = {0};
    HSA_OK(hsa_agent_get_info(ctx.gpu, HSA_AGENT_INFO_NAME, name));
    uint32_t qmax = 0;
    HSA_OK(hsa_agent_get_info(ctx.gpu, HSA_AGENT_INFO_QUEUE_MAX_SIZE, &qmax));
    const uint32_t qsize = qmax < 1024 ? qmax : 1024;

    // Load the code object once; the executable outlives every queue, so a fault
    // cannot be blamed on the kernel image going away.
    hsa_file_t f = open(co_path, 0 /*O_RDONLY*/);
    if (f < 0) { printf("cannot open %s\n", co_path); return 1; }
    hsa_code_object_reader_t reader;
    HSA_OK(hsa_code_object_reader_create_from_file(f, &reader));
    hsa_executable_t exec;
    HSA_OK(hsa_executable_create_alt(HSA_PROFILE_FULL,
                                     HSA_DEFAULT_FLOAT_ROUNDING_MODE_DEFAULT, nullptr,
                                     &exec));
    HSA_OK(hsa_executable_load_agent_code_object(exec, ctx.gpu, reader, nullptr,
                                                 nullptr));
    HSA_OK(hsa_executable_freeze(exec, nullptr));

    hsa_executable_symbol_t sym;
    if (hsa_executable_get_symbol_by_name(exec, "floor_k.kd", &ctx.gpu, &sym) !=
        HSA_STATUS_SUCCESS) {
        // Fall back to the un-suffixed name used by some toolchains.
        HSA_OK(hsa_executable_get_symbol_by_name(exec, "floor_k", &ctx.gpu, &sym));
    }
    uint64_t kernel_object = 0;
    uint32_t kernarg_size = 0, group_size = 0, private_size = 0;
    HSA_OK(hsa_executable_symbol_get_info(
        sym, HSA_EXECUTABLE_SYMBOL_INFO_KERNEL_OBJECT, &kernel_object));
    HSA_OK(hsa_executable_symbol_get_info(
        sym, HSA_EXECUTABLE_SYMBOL_INFO_KERNEL_KERNARG_SEGMENT_SIZE, &kernarg_size));
    HSA_OK(hsa_executable_symbol_get_info(
        sym, HSA_EXECUTABLE_SYMBOL_INFO_KERNEL_GROUP_SEGMENT_SIZE, &group_size));
    HSA_OK(hsa_executable_symbol_get_info(
        sym, HSA_EXECUTABLE_SYMBOL_INFO_KERNEL_PRIVATE_SEGMENT_SIZE, &private_size));

    printf("=== %s ===\n", name);
    printf("queue size %u (max %u), cycles %d, dispatch depth %d\n", qsize, qmax,
           cycles, depth);
    printf("kernel_object 0x%llx kernarg %u group %u private %u\n",
           (unsigned long long)kernel_object, kernarg_size, group_size, private_size);
    printf("Per cycle: hsa_queue_create -> %d dispatches -> doorbell ->\n"
           "hsa_queue_destroy WITHOUT waiting -> free signal and kernarg.\n"
           "Faults, if any, appear in journalctl -k, not here.\n\n",
           depth);

    const size_t karg_bytes = kernarg_size ? kernarg_size : 64;
    int completed = 0;

    for (int c = 0; c < cycles; ++c) {
        hsa_queue_t* q = nullptr;
        hsa_status_t qs = hsa_queue_create(ctx.gpu, qsize, HSA_QUEUE_TYPE_MULTI,
                                           queue_err, nullptr, UINT32_MAX, UINT32_MAX,
                                           &q);
        if (qs != HSA_STATUS_SUCCESS) {
            const char* m = nullptr;
            hsa_status_string(qs, &m);
            printf("cycle %d: hsa_queue_create -> %d (%s) after %d clean cycles\n", c,
                   (int)qs, m ? m : "?", completed);
            break;
        }

        // Fresh signal and kernarg every cycle so the next cycle's allocation can
        // land on the memory this cycle's in-flight packets still reference.
        hsa_signal_t sig{};
        if (hsa_signal_create(1, 0, nullptr, &sig) != HSA_STATUS_SUCCESS) {
            hsa_queue_destroy(q);
            printf("cycle %d: signal_create failed\n", c);
            break;
        }
        void* karg = nullptr;
        if (hsa_amd_memory_pool_allocate(ctx.kernarg, karg_bytes, 0, &karg) !=
            HSA_STATUS_SUCCESS) {
            hsa_signal_destroy(sig);
            hsa_queue_destroy(q);
            printf("cycle %d: kernarg allocate failed\n", c);
            break;
        }
        memset(karg, 0, karg_bytes);
        // The kernarg pool is host memory; the dispatching GPU needs explicit
        // access to it before a wave can read its arguments.
        if (hsa_amd_agents_allow_access(1, &ctx.gpu, nullptr, karg) !=
            HSA_STATUS_SUCCESS) {
            hsa_amd_memory_pool_free(karg);
            hsa_signal_destroy(sig);
            hsa_queue_destroy(q);
            printf("cycle %d: agents_allow_access failed\n", c);
            break;
        }

        const uint64_t base = hsa_queue_add_write_index_screlease(q, depth);
        hsa_kernel_dispatch_packet_t* ring =
            (hsa_kernel_dispatch_packet_t*)q->base_address;

        for (int d = 0; d < depth; ++d) {
            const uint64_t idx = base + d;
            hsa_kernel_dispatch_packet_t* pkt = &ring[idx & (qsize - 1)];
            memset((void*)&pkt->header, 0, sizeof(*pkt));
            pkt->setup = 1 << HSA_KERNEL_DISPATCH_PACKET_SETUP_DIMENSIONS;
            pkt->workgroup_size_x = 64;
            pkt->workgroup_size_y = 1;
            pkt->workgroup_size_z = 1;
            pkt->grid_size_x = 64;
            pkt->grid_size_y = 1;
            pkt->grid_size_z = 1;
            pkt->kernel_object = kernel_object;
            pkt->kernarg_address = karg;
            pkt->group_segment_size = group_size;
            pkt->private_segment_size = private_size;
            // Only the last packet carries the signal, so the queue still has
            // uncompleted packets behind it when we destroy it.
            pkt->completion_signal = (d == depth - 1) ? sig : hsa_signal_t{0};

            uint16_t header = (HSA_PACKET_TYPE_KERNEL_DISPATCH << HSA_PACKET_HEADER_TYPE) |
                              (1 << HSA_PACKET_HEADER_BARRIER) |
                              (HSA_FENCE_SCOPE_SYSTEM
                               << HSA_PACKET_HEADER_SCACQUIRE_FENCE_SCOPE) |
                              (HSA_FENCE_SCOPE_SYSTEM
                               << HSA_PACKET_HEADER_SCRELEASE_FENCE_SCOPE);
            // Publish the header last so the CP cannot see a half-written packet.
            __atomic_store_n((uint16_t*)&pkt->header, header, __ATOMIC_RELEASE);
        }
        hsa_signal_store_screlease(q->doorbell_signal, base + depth - 1);

        // The point of the probe: tear the hardware queue down with dispatches
        // still in flight, then immediately retire what they referenced.
        hsa_queue_destroy(q);
        hsa_signal_destroy(sig);
        hsa_amd_memory_pool_free(karg);
        ++completed;

        if (((c + 1) % 250) == 0) {
            printf("  cycle %d/%d\n", c + 1, cycles);
            fflush(stdout);
        }
    }

    printf("\ncompleted %d cycles of real hardware queue create/destroy with\n"
           "%d dispatches in flight at destroy, signal and kernarg freed\n"
           "immediately after each destroy.\n", completed, depth);
    printf("Check: journalctl -k --since \"5 min ago\" | grep -E "
           "'VM_L2_PROTECTION_FAULT|SQC|REMOVE_QUEUE|sq_intr|reset'\n");

    hsa_executable_destroy(exec);
    hsa_code_object_reader_destroy(reader);
    hsa_shut_down();
    return 0;
}
