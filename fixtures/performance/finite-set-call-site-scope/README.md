# Finite-set call-site scope workload

Saved CLI workload for comparing a narrow finite-set call-site consumer with a
broad call-site consumer while `unique-exports` keeps symbol extraction active
over the full project. It has no Next.js or test-runner setup.

The literal source tree contains 39 TS/TSX files, 3,529 lines, and 212,875
bytes. Thirty-two TSX component modules contribute 3,072 ordinary calls,
JSX, static imports, type imports, and distinct exports. Two scheduler modules
contain 42 `scheduler.register` calls total. The narrow config selects only
the scheduler modules; the broad config contains exact-path specs for all 39
source files. Four helper modules and a shared type provide local imports.

## Configurations

- `consuming.yml`: full-project `unique-exports` plus finite-set call-site
  specs for `worker-a.ts` and `worker-b.ts`. Both contain the same four IDs;
  `minSize: 4` prevents an empty-set comparison from passing accidentally.
- `broad.yml`: the same `unique-exports` check plus explicit finite-set specs
  for all 39 source paths. The worker sets compare equal and each requires four
  values. The other 37 files deliberately contain no `scheduler.register`
  call, so their call-site extraction reports the expected “found no calls
  matching target” findings. Exact paths are required because this extractor
  does not expand file globs. This deliberately failing control requests
  broad call-site facts while keeping the physical source tree unchanged.
- `control.yml`: full-project `unique-exports` without finite-set demand.

The consuming and no-finite-set control reports should have no findings and
exit successfully. The broad control should report 37 no-target findings and
exit with findings. Compare complete JSON reports and exit status between
revisions, including the broad control's expected findings.

## Running the CLI

From the repository root, set `FIXTURE_ROOT` to this directory and run:

```sh
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/consuming.yml" --tsconfig "$FIXTURE_ROOT/tsconfig.json" --format json
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/broad.yml" --tsconfig "$FIXTURE_ROOT/tsconfig.json" --format json
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/control.yml" --tsconfig "$FIXTURE_ROOT/tsconfig.json" --format json
```

## Reproduction and comparison

The checked-in TS/TSX files are literal benchmark inputs. From this directory,
run `python3 scripts/generate.py` to deterministically regenerate the source
tree under `src/`.

Build distinct baseline and candidate binaries from their exact revisions with
identical toolchain, features, and release settings. Freeze and record each
binary's SHA-256 before running the comparison; verify the hashes differ. Use
fresh physical copies of this exact fixture tree for both revisions, and keep
jobs, observer settings, command arguments, and machine constant. Do not compare
one binary against itself or allow a shared build directory to substitute the
wrong executable.
