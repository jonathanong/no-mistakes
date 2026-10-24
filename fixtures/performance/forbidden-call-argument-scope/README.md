# Forbidden-call argument scope workload

Saved CLI workload for measuring the per-file argument-fact demand of
`forbidden-calls` with `whenArguments`. The physical fixture is identical for
all three configurations so baseline and candidate binaries see the same
inventory and source bytes.

The literal source tree contains 39 TS/TSX files, 3,529 lines, and 212,875
bytes. Thirty-two TSX component modules contribute 3,072 helper calls, JSX,
static imports, type imports, and distinct exports. Two scheduler modules each
contain 21 `scheduler.register` calls. Four helper modules and a shared type
provide local imports. The component and helper calls add argument extraction
work without matching the forbidden target.

## Configurations

- `narrow.yml`: one `forbidden-calls` application requests argument facts and
  includes only `src/schedules/**`, excluding `worker-b.ts`. Its roots enumerate
  the complete TS/TSX source tree. `worker-a.ts` contains a matching call with
  `attempt: 0`, so the report has a finding rather than passing empty.
- `broad.yml`: the same argument predicate and root set includes `src/**`,
  requesting argument facts across the complete source tree. The two scheduler
  files contain matching calls.
- `no-arguments.yml`: the same narrow paths and target selector omit
  `whenArguments`, controlling for ordinary forbidden-call work without
  argument-fact demand.

The expected finding counts are 1 for narrow, 2 for broad, and 21 for
no-arguments. Every configuration exits with status 1. The driver asserts both
counts and exit status, then compares complete JSON report hashes and
deterministic output across revisions and repeated runs.

## Running the CLI

From the repository root, set `FIXTURE_ROOT` to this directory and run:

```sh
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/narrow.yml" --format json
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/broad.yml" --format json
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/no-arguments.yml" --format json
```

## Reproduction and comparison

The checked-in TS/TSX files are literal benchmark inputs. From this directory,
run `python3 scripts/generate.py` to deterministically regenerate the source
tree under `src/`.

Build distinct baseline and candidate binaries from their exact revisions with
identical toolchain, features, and release settings. Freeze and record each
binary's SHA-256 before running the comparison; verify the hashes differ. Use
one frozen physical fixture copy for both binaries and fresh invocations. Keep
jobs, observer settings, and machine constant. Do not compare one binary against
itself or allow a shared build directory to substitute the wrong executable.

On Linux, run the saved driver with the two verified binaries and this fixture
copy (choose a fresh output directory):

```sh
python3 scripts/measure.py --baseline /path/to/baseline --candidate /path/to/candidate \
  --fixture "$FIXTURE_ROOT" --out /tmp/forbidden-call-argument-scope-results
```

The driver makes byte-identical executable copies using the same 4 KiB write
sizes, records artifact hashes, raw JSON reports, stderr, wall time, CPU time,
and peak RSS from fresh processes. It tests jobs 1, 4, and default with verbose
timings off and on. Each cell has one warm-up per binary followed by eight
measured samples per binary in balanced ABBA/BAAB order. Startup is recorded
separately and is never subtracted from analysis time. Run without concurrent
builds or tests. All configurations retain 39 TS/TSX parses; the expected
argument fact scope changes from 39 files (broad) to one file (narrow), with
zero files for the no-arguments control.
