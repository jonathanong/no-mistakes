# Call-site assembly workload

Saved source inputs for issue #2635. The corpus has 274 TS/TSX modules,
7,932,529 source bytes, 284,040 lines, and 68 TSX modules. Each of 272 work
modules calls a shared imported helper 1,040 times; TSX modules also expose a
component. The entry module has a reachable `setTimeout` occurrence.

`on.yml` declares thirteen `forbidden-calls` policies over the entire source
inventory. Exactly one policy reports one finding, with exit status 1.
`off.yml` disables the policies over the same physical files and reports no
findings, with exit status 0. `imports.yml` keeps a non-consuming import
graph policy enabled over the same source corpus, with no findings and exit 0.
This is a focused construction workload with
repeated calls and a simple shared import graph, not a full consumer workload.

`generate.py` regenerates the literal saved modules and configurations. Its
source inventory digest includes each sorted module name, a NUL separator,
and its exact bytes. The expected digest is
`9fee7645a1f8ea11421db4929e8ce84a3d5853a7920811b19e517203fe3d6723`.
Do not regenerate source files inside Rust tests.

The CLI snapshot `call_site_assembly_check_json_snapshot` loads this tree and
the smaller semantic fixture through `check_json_snapshot`. It checks all three large
configurations at jobs 1 and 2, including the positive finding's source line.

## Native comparison

Build and freeze distinct baseline and candidate release executables with
identical toolchain, lockfile, default features, fat LTO, and one codegen unit.
Record their exact source revisions and binary hashes. Use an idle machine,
keep the physical fixture unchanged, and run:

```sh
python3 test-cases/performance/call-site-assembly/measure.py \
  --baseline /path/to/baseline --baseline-revision BASE_SHA \
  --candidate /path/to/candidate --candidate-revision CANDIDATE_SHA \
  --rounds 3 --out /tmp/call-site-assembly-results
```

The Linux driver serializes the matrix under `/tmp/nm-heavy.lock`. Each mode
runs at jobs 1, 4, and the available affinity CPU count, pinned to the first
1, 4, or all available CPUs respectively. After a warmup per
binary, balanced blocks alternate baseline/candidate/candidate/baseline and
the reverse ordering. Three blocks give six measured samples per binary per
cell. Every invocation has fresh process-local analysis state. Observer,
progress, and timings flags are omitted; the analyzer deadline is disabled
and lock conflicts fail rather than adding wait time.

Raw records preserve wall seconds, user CPU seconds, system CPU seconds, and
peak RSS in KiB from `/usr/bin/time`. Use `--timings` in a separate output
directory for the diagnostics-on control; default comparisons keep it off. The driver checks exits and positive
finding counts and requires byte-identical complete reports across binaries,
worker settings, warmups, and measured samples for each configuration. It
also rejects equal executable hashes. The rule-off timings include startup;
no startup or reporting time is subtracted from any measurement.
