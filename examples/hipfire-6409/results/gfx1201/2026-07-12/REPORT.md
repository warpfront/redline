# Hipfire/Redline ROCm issue 6409 benchmark

Correctness-gated result: **Redline wins 46/90 rows (51.11%)**.

## Four-way placement table

| Backend | 1st | 2nd | 3rd | 4th | Win % | N |
|---|---:|---:|---:|---:|---:|---:|
| redline | 46 | 44 | 0 | 0 | 51.11 | 90 |
| vulkan | 41 | 39 | 1 | 9 | 45.56 | 90 |
| hipgraph | 2 | 6 | 35 | 47 | 2.22 | 90 |
| hip | 1 | 1 | 54 | 34 | 1.11 | 90 |

## Placement by timing mode

| Mode | Backend | 1st | 2nd | 3rd | 4th | N |
|---|---|---:|---:|---:|---:|---:|
| serial_latency | redline | 14 | 31 | 0 | 0 | 45 |
| serial_latency | vulkan | 28 | 7 | 1 | 9 | 45 |
| serial_latency | hipgraph | 2 | 6 | 32 | 5 | 45 |
| serial_latency | hip | 1 | 1 | 12 | 31 | 45 |
| independent_throughput | redline | 32 | 13 | 0 | 0 | 45 |
| independent_throughput | vulkan | 13 | 32 | 0 | 0 | 45 |
| independent_throughput | hipgraph | 0 | 0 | 3 | 42 | 45 |
| independent_throughput | hip | 0 | 0 | 42 | 3 | 45 |

## Redline losses

| Row | RL place | Beaters (RL slower) | RL us/op | Winner us/op |
|---|---:|---|---:|---:|
| `serial_latency/dispatch-grid/count=50,grid=1` | 2 | vulkan (+35.52%) | 1.7032 | 1.2568 |
| `serial_latency/dispatch-grid/count=200,grid=1` | 2 | vulkan (+48.51%) | 1.5896 | 1.0704 |
| `serial_latency/dispatch-grid/count=941,grid=1` | 2 | vulkan (+55.93%) | 1.5705 | 1.0072 |
| `serial_latency/dispatch-grid/count=64,grid=128` | 2 | vulkan (+154.94%) | 2.9987 | 1.1763 |
| `serial_latency/dispatch-grid/count=64,grid=1024` | 2 | vulkan (+72.71%) | 3.8287 | 2.2169 |
| `serial_latency/reduction/variant=wave,k=8192,rows=1` | 2 | hipgraph (+2.07%) | 136.7469 | 133.9682 |
| `serial_latency/reduction/variant=lds-tree,k=8192,rows=1` | 2 | hip (+0.47%) | 136.9737 | 136.3363 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=4096,body=16` | 2 | vulkan (+21.57%) | 7.4294 | 6.1113 |
| `serial_latency/memory-waitcnt/variant=coalesced4,n=32768,body=16` | 2 | vulkan (+28.30%) | 8.7562 | 6.8250 |
| `serial_latency/memory-waitcnt/variant=gather,n=4096,body=16` | 2 | vulkan (+13.05%) | 10.4031 | 9.2019 |
| `serial_latency/memory-waitcnt/variant=gather,n=32768,body=16` | 2 | vulkan (+22.56%) | 22.8500 | 18.6438 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=4096,body=16` | 2 | vulkan (+20.40%) | 7.5631 | 6.2819 |
| `serial_latency/memory-waitcnt/variant=interleave4,n=32768,body=16` | 2 | vulkan (+62.98%) | 16.3575 | 10.0363 |
| `serial_latency/packed-dot/variant=q8-signed,n=4096,body=16` | 2 | vulkan (+27.25%) | 9.6438 | 7.5788 |
| `serial_latency/packed-dot/variant=q4-unsigned,n=4096,body=16` | 2 | vulkan (+26.93%) | 9.6406 | 7.5950 |
| `serial_latency/packed-dot/variant=q6-zero,n=4096,body=16` | 2 | vulkan (+23.72%) | 9.7919 | 7.9144 |
| `serial_latency/vopd/variant=independent-fma,n=32768,body=64` | 2 | vulkan (+48.44%) | 13.2875 | 8.9512 |
| `serial_latency/vopd/variant=dependent-fma,n=32768,body=64` | 2 | vulkan (+46.15%) | 13.2700 | 9.0800 |
| `serial_latency/vopd/variant=mixed-int-float,n=32768,body=64` | 2 | vulkan (+74.00%) | 28.2300 | 16.2237 |
| `serial_latency/vopd/variant=dequant-like,n=32768,body=64` | 2 | vulkan (+61.94%) | 23.6900 | 14.6288 |
| `serial_latency/sampler/argmax,vocab=32768,rows=1` | 2 | vulkan (+4.50%) | 43.4062 | 41.5369 |
| `serial_latency/sampler/argmax,vocab=32768,rows=4` | 2 | vulkan (+4.62%) | 43.7713 | 41.8381 |
| `serial_latency/sampler/argmax,vocab=131072,rows=1` | 2 | vulkan (+2.84%) | 164.4831 | 159.9437 |
| `serial_latency/sampler/argmax,vocab=131072,rows=4` | 2 | vulkan (+2.89%) | 166.5012 | 161.8300 |
| `serial_latency/two-stage-reduction/k=32768,rows=4,splits=8` | 2 | vulkan (+0.73%) | 10.4850 | 10.4087 |
| `serial_latency/q4-selected-dual/m=512,k=768,tile=2` | 2 | vulkan (+22.53%) | 7.3750 | 6.0187 |
| `serial_latency/q4-selected-dual/m=2048,k=2048,tile=2` | 2 | vulkan (+4.10%) | 14.3387 | 13.7737 |
| `serial_latency/q4-selected-dual/m=4096,k=8192,tile=2` | 2 | vulkan (+47.96%) | 88.3600 | 59.7200 |
| `serial_latency/q6-x8-selected-down/m=512,k=2048,tile=8` | 2 | hipgraph (+0.88%) | 37.1213 | 36.7988 |
| `serial_latency/dense-q8/m=512,k=768,tile=4` | 2 | vulkan (+13.86%) | 9.5162 | 8.3575 |
| `serial_latency/dense-q8/m=2048,k=2048,tile=4` | 2 | vulkan (+0.04%) | 19.3575 | 19.3500 |
| `independent_throughput/dispatch-grid/count=64,grid=1024` | 2 | vulkan (+54.68%) | 2.2506 | 1.4550 |
| `independent_throughput/dispatch-grid/count=64,grid=8192` | 2 | vulkan (+49.85%) | 8.2719 | 5.5200 |
| `independent_throughput/memory-waitcnt/variant=gather,n=32768,body=16` | 2 | vulkan (+23.59%) | 9.4887 | 7.6775 |
| `independent_throughput/memory-waitcnt/variant=interleave4,n=32768,body=16` | 2 | vulkan (+21.32%) | 5.1562 | 4.2500 |
| `independent_throughput/packed-dot/variant=q8-signed,n=4096,body=16` | 2 | vulkan (+6.23%) | 1.4606 | 1.3750 |
| `independent_throughput/packed-dot/variant=q4-unsigned,n=4096,body=16` | 2 | vulkan (+1.52%) | 1.4169 | 1.3956 |
| `independent_throughput/vopd/variant=independent-fma,n=32768,body=64` | 2 | vulkan (+8.61%) | 4.4013 | 4.0525 |
| `independent_throughput/vopd/variant=dependent-fma,n=32768,body=64` | 2 | vulkan (+8.12%) | 4.3262 | 4.0012 |
| `independent_throughput/vopd/variant=mixed-int-float,n=32768,body=64` | 2 | vulkan (+40.99%) | 10.4863 | 7.4375 |
| `independent_throughput/vopd/variant=dequant-like,n=32768,body=64` | 2 | vulkan (+21.17%) | 8.0425 | 6.6375 |
| `independent_throughput/q4-selected-dual/m=2048,k=2048,tile=2` | 2 | vulkan (+2.49%) | 3.9100 | 3.8150 |
| `independent_throughput/q6-x8-selected-down/m=4096,k=8192,tile=8` | 2 | vulkan (+14.88%) | 28.6550 | 24.9425 |
| `independent_throughput/dense-q8/m=2048,k=8192,tile=4` | 2 | vulkan (+1.60%) | 12.7050 | 12.5050 |

## Pairwise Redline results

| Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---:|---:|---:|---:|---:|
| RL / vulkan | 49 | 41 | 54.44 | 0.9538 | 90 |
| RL / hipgraph | 88 | 2 | 97.78 | 0.2119 | 90 |
| RL / hip | 89 | 1 | 98.89 | 0.2144 | 90 |

## Prior hipEngine-harness baseline

The old run is directional rather than row-for-row identical: it used a larger parameter sweep, only three contenders, and the previous kernels.

| Harness | Comparison | Wins | Losses | Win % | Median ratio | N |
|---|---|---:|---:|---:|---:|---:|
| hipEngine | RL / vulkan | 186 | 26 | 87.74 | 0.4788 | 212 |
| hipEngine | RL / hip | 157 | 55 | 74.06 | 0.8342 | 212 |

## Harness verdict

This is **not a Hipfire harness failure**: Redline beats direct HIP in 89/90 rows and HipGraph in 88/90 while all three load the identical HSACO. The remaining crossover is real but has two distinct sources. Serial no-op and RMW losses expose Redline's current full-idle/full-acquire dependency cost; Vulkan-only VOPD, packed-dot, memory, and large-Q4 wins expose compiler/lowering or kernel-scheduling differences because submission cannot change the shared HIP ISA. Redline still beats Vulkan pairwise in 49/90 rows, so neither the categorical ‘HIP is inherently slower’ theory nor the categorical ‘it was all hipEngine’ theory survives this control.

## Interpretation guardrails

HIP, HipGraph, and Redline load the identical hipcc code object; their differences isolate launch/submission and dependency handling. Vulkan uses matched GLSL compiled by the Mesa stack, so Vulkan-only wins can still include compiler scheduling and ISA differences. All placement counts exclude any row where one of the four outputs failed the CPU oracle.
