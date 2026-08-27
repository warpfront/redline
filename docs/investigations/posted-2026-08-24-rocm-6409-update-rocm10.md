Update: I've re-run all of the above on **ROCm 10.0.0**, installed from `stable.repo.amd.com` today, side by side with 7.14 on the same hosts so the only variable is the runtime. HIP reports `7.15.26333-0000000` inside ROCm 10.0.0.

**The dispatch floor is unchanged.** Median of 200 replays after 20 warmups, N=512, µs per dispatch, both runtimes on the same host and GPU:

| GPU | arm | ROCm 7.14 | ROCm 10.0 |
| --- | --- | ---: | ---: |
| gfx1201 (R9700) | stream-loop | 2.567 | 2.551 |
| | per-launch-sync | 18.523 | 18.520 |
| | **graph-replay** | **2.147** | **2.147** |
| gfx1100 (7900 XTX) | stream-loop | 3.129 | 3.132 |
| | **graph-replay** | **2.815** | **2.818** |
| gfx1151 (Strix Halo) | stream-loop | 1.835 / 1.837 / 1.842 | 1.888 / 1.886 / 1.840 |
| | **graph-replay** | **1.752 / 1.751 / 1.752** | **1.750 / 1.751 / 1.750** |

Graph replay is identical to three decimal places on all three architectures. The gfx1151 rows are three separate runs each because a single 10.0 run initially read 2.141 on the stream-loop arm; repeating it shows that was an outlier and the two runtimes overlap on that arm. Every row printed `agree` on the two independent clocks and passed its correctness gate.

Two notes on why I checked:

- ROCm 10.0.0's release notes state that "Event operations are now coalesced to eliminate redundant barrier submissions." That is a real change, but as the note says it is scoped to *event* operations — it does not move the kernel-dispatch floor measured here.
- Taken with the ROCm 7.2 figure quoted earlier in my previous comment (real `hipGraphLaunch` at 2.113–2.133 µs/dispatch on the same GPU model), the per-dispatch floor is now unchanged across **ROCm 7.2 → 7.14 → 10.0**.

So the numbers in my previous comment are current as of the latest release, not stale. Same reproducer, same build command; it compiles unmodified against the 10.0 toolchain (AMD clang 23.0.0git).

Happy to re-run on any other configuration that would be useful.
