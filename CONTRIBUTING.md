# Contributing

This repository is a huge token sink. Thus, contributions are welcomed.

1. Please add test cases as files on disk, never built inside a test:
   - New scenarios go in `test-cases/<category>/<scenario>/fixture/`. Rule pass/fail pairs go in
     `test-cases/rules/<rule>/fixture/{pass,fail}`.
   - A suite whose loader reads `fixtures/<category>/<name>/` (benchmarks, N-API, test-plan, and
     some per-rule cases) keeps its cases there. Both trees hold some `rules/`, `lockfile/`, and
     `check-runner/` cases, so find the suite's loader with `rg` and add to the tree it reads.
2. Rule output tests commit file snapshots. Review `.snap` and Vitest file-snapshot
   diffs like code. Rust `check --format json` snapshots use
   `crates/no-mistakes/tests/common/check_json_snapshot.rs`, which compares `--jobs 1`
   with `--jobs 2` and rewrites absolute paths, temp directories, process ids, and
   timings so the file is stable on Linux, macOS, and Windows. CI sets `INSTA_UPDATE=no`
   and fails when a snapshot is missing or changed. Regenerate on purpose with
   `INSTA_UPDATE=1 cargo test -p no-mistakes --test suite_cli_rules check_json_snapshot`
   or `cargo insta review`. In `packages/eslint-plugin-no-mistakes`, fixture
   diagnostics use Vitest `toMatchFileSnapshot` with relative paths and sorted
   findings. CI does not pass `-u`. Update with `pnpm exec vitest run <test> -u`
   only when that output change is intended.
3. Annotate which AI harness + model was used, `Co-Authored-By` is preferred
4. Maintain 98% project and patch test coverage

<!-- cspell:ignore rustup Clippy -->
Development, CI, release builds, and benchmarks use the latest stable Rust.
Run `rustup update stable` before local validation; `rust-toolchain.toml` selects
that channel and installs Clippy and rustfmt. Workflows select `toolchain: stable`
while pinning the setup action code separately. Benchmark comparisons must use
the same resolved compiler version for both runs and record that version.

CI enforces per-function CRAP and cyclomatic complexity limits of 30. The
complexity gate runs `cargo crap --workspace --missing optimistic --fail-above
--threshold 30 --summary` without LCOV: assuming full coverage makes CRAP equal
cyclomatic complexity, including source omitted from the coverage report.

Run `pnpm run lint:duplicates` to check for copied blocks of at least 30 lines
and functions with at least 80% structural similarity. CI runs the same jscpd
check and fails on any reported clone. See the [duplication guide](docs/development/code-duplication.md)
for its scope and how to fix findings.

Native CI builds and platform tests omit Rust debug metadata to keep compilation
within their ten-minute step deadlines. Development and test profiles use the same
setting so Windows can reuse the CLI's library build. Debug assertions stay enabled;
Linux still runs the full suite and enforces the coverage gates. These defaults
live in the workspace Cargo profiles, including release validation. For local
debugging, use `CARGO_PROFILE_DEV_DEBUG=2` or `CARGO_PROFILE_TEST_DEBUG=2`.

CI and release validation share `.github/scripts/rust-validation.sh`: `build` and
`test` select the locked workspace, all features, libraries, binaries, and integration
tests; `doc` runs documentation tests separately. Coverage instrumentation stays scoped to
the coverage build and test steps. Run the shared script locally with Bash to
reproduce either validation scope. Release resumes use validation tooling from
the running workflow while retaining the tagged Rust source; see the
[CI build guide](docs/development/ci-builds.md).

Rust integration tests share five explicitly registered suites: `suite_cli_queries`,
`suite_cli_rules`, `suite_cli_postgres`, `suite_cli_test_plans`, and
`suite_contracts`. This links the analyzer once per suite instead of once per
source file. Keep new root `crates/no-mistakes/tests/*.rs` modules registered in
one suite; the registration test rejects omitted or duplicate sources. Original
module and fixture paths stay unchanged. To run one former target, use, for example,
`cargo test --locked -p no-mistakes --test suite_cli_queries cli_queries::`.
Use the full module-qualified name with `-- --exact` for a single test.
`windows_job_object` and `macos_invocation` remain separate platform harnesses
for their process and crate-root requirements.

For this repository, use `no-mistakes impacted-checks <changed-file> --root .
--format json` to obtain the configured Rust validation command. The Cargo test
planner currently derives integration target names from source filenames; it
cannot project a changed suite module onto an explicit `[[test]]` target.
Run the returned workspace test command or the suite commands above.

CI checks links to this repository's `main` documentation against the checked-out
`docs/` files, including new pages in a pull request. Missing targets still fail;
links outside that documentation prefix continue to be checked over the network.

ARM macOS release builds use ThinLTO and 16 code generation units so LLVM can
optimize the combined CLI and N-API build in parallel on a cold runner. The
workflow sets these Cargo profile overrides before restoring its build cache.
Other release targets and local builds retain full LTO and one code generation
unit. Compile-time and runtime differences require measurements on the same
platform; Linux benchmarks do not establish the macOS tradeoff.
