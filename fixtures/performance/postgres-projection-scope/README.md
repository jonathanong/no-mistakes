# PostgreSQL projection scope benchmark

`src/components` contains 32 TSX modules, each with 96 JSX rows and 16 unique
SQL queries (512 calls total). The `narrow.yml` workload selects only
`component-00.tsx`; its last query contains the one expected OFFSET finding.
The `broad.yml` control analyzes all 32 modules and still produces that one
finding. The `no-postgres.yml` control runs `unique-exports` over the same
source inventory so shared TS facts stay active without PostgreSQL checks.

Each component imports `../db.mjs`; the only source module is `src/db.mts`.
The explicit `nodenext` resolver configuration exercises `.mjs` to `.mts`
source remapping. Successful executor recognition and identical one-finding
reports in narrow and broad configs verify this import path.

Run `python3 generate.py` to regenerate the saved TSX and `.mts` files. The
benchmark driver consumes the saved fixture and does not create input sources.

## Run the benchmark

Build distinct baseline and candidate binaries from their exact revisions with
the same toolchain, features, and release settings. Then run:

```sh
python3 scripts/measure.py \
  --baseline /path/to/baseline/no-mistakes \
  --candidate /path/to/candidate/no-mistakes \
  --fixture "$PWD/fixtures/performance/postgres-projection-scope" \
  --out /tmp/postgres-projection-scope-results
```

The driver verifies distinct binary hashes and freezes both binaries using the
same 4 KiB copy sizes. It sets a private temporary runtime directory, clears
`RAYON_NUM_THREADS`, uses explicit `--root`, and runs startup separately from
analysis. Each workload runs with jobs 1, 4, and default, diagnostics off and
on, one warm-up per binary, and eight measured samples per binary in balanced
ABBA/BAAB order. It saves raw JSON, stderr, wall and CPU times from
`perf_counter`/`wait4`, peak RSS, expected findings, exit status, and report
hashes. Expected outcomes are narrow: one finding / exit 1; broad: one finding
/ exit 1; no-Postgres: zero findings / exit 0. Every report hash and exit code
must remain identical across all samples and both binaries.
