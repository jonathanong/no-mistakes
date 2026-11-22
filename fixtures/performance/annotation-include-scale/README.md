# Annotation include scale workload

This saved workload contains 3,072 small TypeScript modules that import the same synthetic `@bench/db` client. Each module has conditional control flow, calls direct and factory-created typed executors, and imports the shared provider. The client declares two configured executor factories (`createReader`, `createWriter`) and four configured executor types (`Reader`, `Writer`, `Replica`, `Archive`). The excluded runtime importer calls the one finding-bearing module from outside its include scope; the shared provider and cycle modules are also outside the configured rule includes. SQL calls are annotated except for the expected finding in `src/module-0007.mts`.

All includes resolve from this same fixture root. `tsconfig.json` resolves `@bench/db` to `src/db.mts`. The full and small configs explicitly set Max or Low effort. `small-max.yml` contains two same-profile Max entries whose disjoint includes together select the same 32 modules as `small-low.yml`; `half-a-16-max.yml` and `half-b-16-max.yml` provide disjoint-half controls.

The module selection constants are deliberate to keep query reports stable. Both AST arms are lowered, so this measures parsing and lowering rather than runtime execution coverage.

From the repository root, run the same commands on baseline and candidate revisions, then compare normalized JSON reports including findings, warnings, advisories, and exit status:

```sh
cd fixtures/performance/annotation-include-scale
no-mistakes check --root "$PWD" --config configs/full-max.yml --jobs 8 --timeout 0 --format json
no-mistakes check --root "$PWD" --config configs/small-max.yml --jobs 8 --timeout 0 --format json
no-mistakes check --root "$PWD" --config configs/full-low.yml --jobs 8 --timeout 0 --format json
no-mistakes check --root "$PWD" --config configs/small-low.yml --jobs 8 --timeout 0 --format json
no-mistakes check --root "$PWD" --config configs/half-a-16-max.yml --jobs 8 --timeout 0 --format json
no-mistakes check --root "$PWD" --config configs/half-b-16-max.yml --jobs 8 --timeout 0 --format json
```

`--timeout 0` disables the command deadline; `--lock-timeout 0` disables the invocation-lock wait deadline. `measure.py` passes both flags.

`measure.py BASELINE CANDIDATE OUTPUT` runs balanced baseline/candidate/candidate/baseline release comparisons. Each invocation starts with fresh in-memory analysis state. It checks exact report bytes and status, and records wall time, user plus system CPU, and peak RSS. Acceptance runs have no observer flags. Separate `--timings` runs attribute the parse phase; those instrumented runs do not supply the acceptance wall/CPU measurements.

Use `--jobs 1`, `--jobs 4`, and `--jobs 0` for single-worker, four-worker, and default-worker memory controls. `measure.py` defaults to 8 workers; `--jobs 0` omits the child CLI flag and uses its logical-CPU default. `RAYON_NUM_THREADS` is removed from each child environment.

This fixture records workload shape only and makes no runtime-performance claim. Regenerate with `python3 materialize.py`.
