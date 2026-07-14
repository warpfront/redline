# Low-margin Vulkan loser tuning

This pass started from the controlled `targeted_wave64` result and tuned the
closest reproducible Vulkan losses first.  The two closest nominal losses were
not tuning targets: sampler `vocab=131072,rows=4` (+0.06%) and the large
two-stage serial row (+0.11%) both changed sign between the original two
replicates.

The first stable kernel-side losses were aggressive gather (+2.25%),
independent packed Q8 (+3.21%), and independent packed Q4 (+5.14%).

## Accepted result

The accepted kernel uses ordinary ROCm 7.2 hipcc:

- gfx12 `buffer_load_b32`/`buffer_store_b32` builtins replace 64-bit global
  address construction in gather and packed-dot kernels;
- the `body=16` gather fast path exposes four independent index/data chains;
- packed dots retain one accumulator after a four-accumulator experiment was
  measured and rejected.

Two full 133-row passes used three warmups and seven measured samples per
backend.  Every backend/row passed its CPU oracle.  Each aggregate below is the
median of all 14 raw samples.

| Kernel state | RL first | Strict RL>Vulkan | Median RL/Vulkan |
|---|---:|---:|---:|
| controlled targeted-wave64 baseline | 94/133 | 94 | 0.8485x |
| tuned buffer-addressed HIP | **98/133** | **98** | **0.8201x** |

| Mode | Baseline RL wins | Tuned RL wins | Tuned median RL/Vulkan |
|---|---:|---:|---:|
| serial RMW | 22/45 | **23/45** | 0.9710x |
| independent | 36/45 | **38/45** | 0.6188x |
| aggressive N=1 | 36/43 | **37/43** | 0.8051x |

The raw passes are in [`full-buffer/rep1`](full-buffer/rep1/results.json) and
[`full-buffer/rep2`](full-buffer/rep2/results.json); the combined result is
[`full-buffer/aggregate.json`](full-buffer/aggregate.json).

## Causal crossings

Negative tuned margins mean Redline is faster than Vulkan.

| Row | Baseline RL behind VK | Tuned margin | RL time change |
|---|---:|---:|---:|
| aggressive gather, n=32768 | +2.25% | **-0.60%** | **-3.08%** |
| independent packed Q8 | +3.21% | **-33.22%** | **-35.78%** |
| independent packed Q4 | +5.14% | **-33.67%** | **-37.26%** |
| serial gather, n=4096 | +12.18% | **-37.88%** | **-45.24%** |

The unrelated large-sampler aggressive rows moved by less than 0.7% and
flipped in opposite directions.  They are noise and cancel in the net count;
the four-placement aggregate gain above comes from the four tuned rows.

Packed serial latency also moved close to parity: Q6 went from +23.44% behind
Vulkan to +1.27%, Q8 from +27.07% to +3.66%, and Q4 from +27.67% to +4.00%.
Their aggressive N=1 rows remain 19-21% faster than Vulkan, so the residual
serial gap is the per-dispatch RMW dependency cost rather than deficient dot
throughput.

## ISA evidence

Before tuning, hipcc emitted gather as a pair of `global_load_b32` operations
with repeated 64-bit address construction.  Packed Q8 occupied 26 VGPRs for
the same reason.  RADV/ACO used 32-bit byte offsets and buffer descriptors.

After tuning:

| Kernel | Wave | VGPR | SGPR | Private bytes | Spills |
|---|---:|---:|---:|---:|---:|
| gather | 32/64 | 8 | 21 | 0 | 0 |
| packed Q8/Q4/Q6 | 32/64 | 19 | 20 | 0 | 0 |
| scalar Q4 | 32 | 67 | 20 | 0 | 0 |
| scalar Q4 | 64 | 45 | 20 | 0 | 0 |

This is still HIP source compiled by hipcc.  The source requests AMD buffer
operations through compiler builtins; it does not embed ACO ISA or launch a
Vulkan kernel through Redline.

## Rejected variants

- Fully unrolling all 16 global gather chains used 39 VGPRs.  It erased the
  aggressive gap within noise but made independent throughput about 4% worse.
- Rolling global-load windows used 15 VGPRs at width four and 22 at width
  eight.  Four was the better balance, but neither matched buffer addressing.
- Four independent packed-dot accumulators used 22 rather than 19 VGPRs.  A
  second pair of full sweeps reported 99/133, but Redline packed-dot time was
  0.68% slower geometrically across all 12 packed rows.  The extra placement
  came from unrelated sampler/two-stage noise, so the change was reverted.

## Next low-margin work

The accepted aggregate's closest losses are a +0.65% large-sampler N=1 row
and a +1.15% two-stage serial row; both already flipped sign in other full
passes.  The first remaining stable kernel/fence cluster is serial packed Q6
(+1.27%), then the large sampler serial rows (+3.16% to +4.47%) and serial
packed Q8/Q4 (+3.66%/+4.00%).  Packed N=1 is already decisively faster, so the
next causal lever there is a cheaper correct Redline RMW dependency packet,
not more dot unrolling.
