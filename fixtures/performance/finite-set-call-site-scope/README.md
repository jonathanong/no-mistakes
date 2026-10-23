# Finite-set call-site scope workload

Saved CLI workload for comparing a finite-set consumer over two call-source
files while `unique-exports` keeps symbol extraction active over the full
project. It has no Next.js or test-runner setup.

The literal source tree contains 39 TS/TSX files, 3,527 lines, and 212,923
bytes. Thirty-two TSX component modules contribute 3,072 ordinary calls, JSX,
static imports, type imports, and distinct exports. Two scheduler modules add
40 calls; only these two files are configured for `ts-call-first-string-argument`.
Four helper modules and a shared type provide local import resolution. The
source layout intentionally makes the call-site demand a small subset of the
files already needed for symbol and import analysis.

## Configurations

- `consuming.yml`: full-project `unique-exports` plus two finite-set call-site
  specifications. The sets match and should produce no findings.
- `control.yml`: full-project `unique-exports` without finite-set call-site
  demand. It should also produce no findings.

Run the public `check` command from the repository root, with `FIXTURE_ROOT`
set to this directory:

```sh
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/consuming.yml" --format json
no-mistakes check --root "$FIXTURE_ROOT" --config "$FIXTURE_ROOT/control.yml" --format json
```

Both reports should have no findings. The consuming run must include the
`finite-set-consistency` rule; the control run must include `unique-exports`.
Use identical binaries, jobs, observer settings, and fixture copies when
comparing revisions. The configs are saved as separate files so the control
does not rely on editing config between samples.

## Regeneration

The checked-in source files are literal benchmark inputs. To reproduce them
from this directory, run:

```sh
python3 scripts/generate.py
```

The generator is deterministic and writes only the source tree under `src/`.
