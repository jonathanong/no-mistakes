<!-- cspell:ignore mimalloc -->

Scoped allocator observations only; #2200 remains a draft, with no adoption claim.

Native binaries, sampler and fixture checkout: `8c4ca1d58a6d0322780333c743a2cf98f5e3b80f`.

Four matched fresh-process pairs per cell, ordered AB/BA/BA/AB. Each resource entry is the median of four native processes; CPU is user + system CPU. RSS is the per-process peak, reported as median [largest observed], in MiB. Paired changes are the median of four adjacent mimalloc/system ratios, independently for wall, CPU and RSS; they need not equal ratios of the resource medians.

These are descriptive finite samples, not a confidence bound, tail-latency claim or full-config pass. The 131-rule cohort removes exactly the two annotation applications from the pinned 133-rule consumer; no other configuration payload changes. The separate saved annotation workload has one annotation rule, 32 roots and 34 TS files. It demonstrates worker scaling for this fixture only. Short annotation wall times include the 50 ms process polling granularity.

The failed original Linux setup produced zero native samples and contributes no measurements. Linux retry 39474778706 uses the immutable original 39467428353 build artifacts and exact original source checkout; macOS completed in the original run. Workflow-only retry source is 04b8501eb0295d16d14b16862166185aa1c780ef. Both result origins, all native artifact identities, worker probes, source/config/inventory hashes, complete JSON/status/stderr parity, and all 160 raw processes were independently checked.

Reviewed 131-rule cohort

| Platform | Diagnostics | Jobs (effective) | System: wall s / CPU s / RSS MiB [max] | Mimalloc: wall s / CPU s / RSS MiB [max] | Paired change: wall / CPU / RSS |
| --- | --- | --- | ---: | ---: | ---: |
| linux-arm64 | off | 1 (1) | 92.759 / 92.501 / 5706.02 [5715.40] | 70.520 / 70.508 / 5801.23 [5803.08] | -23.60% / -23.41% / +1.65% |
| linux-arm64 | off | 4 (4) | 45.404 / 89.302 / 6916.12 [6951.94] | 36.426 / 71.657 / 5921.00 [5931.01] | -19.77% / -20.15% / -14.52% |
| linux-arm64 | off | max (4) | 47.234 / 89.182 / 6647.86 [6901.81] | 37.022 / 70.855 / 5940.43 [5977.12] | -21.04% / -21.03% / -11.02% |
| linux-arm64 | on | 1 (1) | 92.599 / 92.588 / 5719.09 [5733.41] | 70.162 / 70.130 / 5813.54 [5815.79] | -24.36% / -24.38% / +1.65% |
| linux-arm64 | on | 4 (4) | 45.015 / 90.122 / 6899.62 [7005.05] | 37.756 / 72.484 / 5937.94 [5952.72] | -16.11% / -19.79% / -13.94% |
| linux-arm64 | on | max (4) | 45.118 / 88.937 / 7006.34 [7037.36] | 38.306 / 71.824 / 5927.06 [5963.36] | -14.91% / -19.58% / -15.44% |
| macos-arm64 | off | 1 (1) | 127.570 / 117.237 / 3443.96 [3469.83] | 129.669 / 114.540 / 3517.72 [3677.86] | +0.35% / -3.16% / +4.43% |
| macos-arm64 | off | 4 (4); exceeds automatic worker count | 75.451 / 121.057 / 3696.52 [3776.08] | 68.384 / 101.202 / 3772.86 [3852.03] | -10.63% / -18.99% / +2.59% |
| macos-arm64 | off | max (3) | 86.988 / 132.695 / 3615.97 [3730.23] | 80.356 / 120.036 / 3675.37 [3727.56] | -8.27% / -10.11% / +0.57% |
| macos-arm64 | on | 1 (1) | 118.063 / 110.240 / 3765.06 [3888.73] | 110.521 / 102.081 / 3857.22 [3958.22] | -4.62% / -5.24% / +1.40% |
| macos-arm64 | on | 4 (4); exceeds automatic worker count | 71.171 / 119.079 / 3833.16 [3842.58] | 83.699 / 125.789 / 3907.38 [3947.03] | +10.88% / +3.43% / +2.11% |
| macos-arm64 | on | max (3) | 86.504 / 139.730 / 3753.88 [3819.25] | 71.985 / 112.317 / 3895.87 [3992.42] | -8.66% / -12.07% / +4.78% |

Completing annotation fixture

| Platform | Diagnostics | Jobs (effective) | System: wall s / CPU s / RSS MiB [max] | Mimalloc: wall s / CPU s / RSS MiB [max] | Paired change: wall / CPU / RSS |
| --- | --- | --- | ---: | ---: | ---: |
| linux-arm64 | off | 1 (1) | 4.318 / 4.287 / 28.39 [28.39] | 3.873 / 3.851 / 43.68 [43.69] | -10.32% / -10.09% / +53.89% |
| linux-arm64 | off | max (4) | 1.155 / 4.299 / 32.19 [32.34] | 1.055 / 3.876 / 92.58 [99.52] | -8.69% / -9.84% / +186.54% |
| linux-arm64 | on | 1 (1) | 4.322 / 4.304 / 28.64 [28.64] | 3.866 / 3.849 / 50.76 [51.35] | -10.50% / -10.51% / +77.61% |
| linux-arm64 | on | max (4) | 1.157 / 4.307 / 32.59 [32.66] | 1.055 / 3.877 / 89.63 [91.71] | -8.82% / -9.99% / +175.45% |
| macos-arm64 | off | 1 (1) | 3.298 / 3.174 / 31.57 [32.25] | 3.104 / 2.980 / 31.69 [31.70] | -12.63% / -10.45% / +0.33% |
| macos-arm64 | off | max (3) | 1.316 / 3.505 / 33.06 [33.38] | 1.271 / 3.079 / 38.77 [39.62] | -5.79% / -12.78% / +16.75% |
| macos-arm64 | on | 1 (1) | 2.856 / 2.820 / 32.23 [32.28] | 2.705 / 2.611 / 33.93 [34.16] | -8.49% / -7.98% / +5.55% |
| macos-arm64 | on | max (3) | 1.357 / 3.546 / 33.85 [33.94] | 1.316 / 3.061 / 38.66 [39.56] | -2.20% / -14.03% / +14.46% |

Annotation worker-count RSS scaling (descriptive; median peak MiB)

| Platform | Diagnostics | Automatic workers | System: jobs 1 → max (ratio) | Mimalloc: jobs 1 → max (ratio) |
| --- | --- | ---: | ---: | ---: |
| linux-arm64 | off | 4 | 28.39 → 32.19 (1.134×) | 43.68 → 92.58 (2.119×) |
| linux-arm64 | on | 4 | 28.64 → 32.59 (1.138×) | 50.76 → 89.63 (1.766×) |
| macos-arm64 | off | 3 | 31.57 → 33.06 (1.047×) | 31.69 → 38.77 (1.224×) |
| macos-arm64 | on | 3 | 32.23 → 33.85 (1.050×) | 33.93 → 38.66 (1.139×) |
