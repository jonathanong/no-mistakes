<!-- cspell:ignore BAAB maxrss stationarity codegen EACCES -->

# Hosted historical receiver evidence preservation

Refs #1972 and #2012. This package preserves the exact readable artifact from [run 38654992458](https://github.com/jonathanong/no-mistakes/actions/runs/38654992458), experiment source `b1a348178fe3282663b7198e08f36bdcd60817e1`. It compares historical `57c927df4c09f450f67248277d49070f9576ba3b` against `8d81aa148eb2e62773919a290dd21439a1616f42`, not current main or the merged integration comparison.

The artifact ZIP is retained privately byte for byte: 21,798 bytes, SHA-256 `3848786698e7834b61b1a40fcf32450d4710c5e0918d8b87fee8619df2ece808`. Its API expiry is 2026-10-21T23:35:14Z. The public copy contains only the 17 original readable JSON/Markdown files, including the checksum manifest; no ZIP, executables, libraries, raw build/cache logs or profiler payloads. All 16 manifest entries match their exact byte counts and SHA-256 hashes. No source measurement record was transformed.

Independent validation checked all 288 rows: 64 original-100 rows, 16 pilot rows and 64 long rows for each profile. Both pilots met the preset criteria; long batch counts were 3,000 collections for opt3 and 500 for opt0. Every row passes balanced order/block/arm checks, exit/signature hashes, user+system CPU accounting, fixed CPU affinity [0,1,2,3] and four Rayon workers. Both arms have exact historical revisions/trees, identical lockfile/default features/fixture/probe hashes and distinct freshly compiled library/binary hashes. Full bootstrap reassessment and a separately implemented paired-mean calculation agree with the saved assessment.

| Profile/demand | Long paired CPU change | Descriptive 95% interval | Interpretation |
| --- | ---: | --- | --- |
| opt3 values | -1.219% | [-1.916%, -0.529%] | Lower cost observed in this run |
| opt3 imports | -0.895% | [-2.042%, +0.287%] | Inconclusive |
| opt0 values | -1.107% | [-2.598%, +0.173%] | Inconclusive |
| opt0 imports | -0.021% | [-0.566%, +0.548%] | Inconclusive |

The shorter pilots point in the opposite direction for both optimized demands and opt0 values. This direction reversal limits any general conclusion from the long series:

| Profile/demand | Pilot collections | Paired CPU change | Descriptive 95% interval |
| --- | ---: | ---: | --- |
| opt3 values | 1,000 | +0.844% | [+0.688%, +1.001%] |
| opt3 imports | 1,000 | +0.695% | [+0.236%, +1.157%] |
| opt0 values | 100 | +1.274% | [+1.084%, +1.465%] |
| opt0 imports | 100 | -0.945% | [-2.162%, +0.287%] |

Pilot intervals resample only two balanced block ratios (one ABBA and one BAAB), versus eight for the long series; they are especially limited descriptive diagnostics, not confirmation of a stable effect. Different batch sizes are reported separately without carrying effect percentages between them.

Long-series measured wall time and peak RSS remain separated by demand and arm. Wall is the mean seconds per whole child batch, including startup; peak RSS is the maximum wait4 ru_maxrss among that arm/demand's 16 children, in KiB. No wall or memory causal attribution is implied.

| Profile/demand | Baseline mean wall (s) | Candidate mean wall (s) | Baseline peak RSS (KiB) | Candidate peak RSS (KiB) |
| --- | ---: | ---: | ---: | ---: |
| opt3 values | 5.358242 | 5.292661 | 19,220 | 19,144 |
| opt3 imports | 5.204641 | 5.158269 | 19,432 | 18,632 |
| opt0 values | 5.668327 | 5.605138 | 68,148 | 67,724 |
| opt0 imports | 5.562667 | 5.561643 | 67,432 | 65,916 |

Each demand/profile is assessed independently. The opt3 aggregate label does not establish the imports result. Intervals describe the eight balanced block ratios on one hosted VM; they do not guarantee stationarity or dedicated physical CPU access. Original-100 and pilot rows remain alongside the long rows and are not pooled, discarded or used for startup subtraction. Original-100 paired CPU changes were opt3 values +0.164%, imports +0.290%, and opt0 values +0.199%, imports +0.639%.

The runner was x86_64 Ubuntu 24, AMD EPYC 9V45, four allowed CPUs, with Rust 1.99.0. Opt3 uses full LTO and one codegen unit; opt0 is the separately labeled original dev/debug-zero control. PMU capability was unavailable (`EACCES`, paranoid 4); these are native timing results, not retired-instruction attribution.

The long series did not establish positive optimized cost on this fixture; the shorter pilots observed positive cost in both optimized demands, so the combined run does not establish a stable direction or a general speedup. This does not establish zero overhead, explain the original cloud-host opt0 values +5.42%/+7.79% and imports +4.70%/+6.28% observation, or measure other fixtures and the merged b97-to-917 integration pair. Original historical raw evidence remains unchanged. #1972 stays open; no attributable runtime fix or resolution is claimed.
