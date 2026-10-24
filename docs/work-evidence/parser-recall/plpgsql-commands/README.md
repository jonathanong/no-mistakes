# PL/pgSQL command recovery review delta

This extends PR #2682 after reviewed head `ec133f2ed668597c053b386fbe452ff9bf077c6c`.
The implementation and saved-fixture revision is `45c8de326`; source fingerprints
cover the complete delta from that reviewed head. No public API fields change.

The procedural corpus now includes declarations after the first variable,
assignments with either operator, record/array targets, diagnostics, cursor
options, SELECT/EXECUTE INTO, nested declarations, loops and exception handlers.
Seven positive saved fixtures were accepted by PostgreSQL 18 in an isolated
instance. Unknown SQL after blocks and bodies retains its own source line;
malformed command payloads and lexical failures remain reported. Query-call
fixtures lock 32 source-line occurrences, including same-line calls around a
WITH query and calls inside data-modifying CTEs, without duplicate facts.

The matched default-CLI comparison uses the frozen reviewed binary and current
candidate against the same 109 saved configurations. 108 outputs are byte-identical.
Only the expanded procedural corpus changes: 172 findings become 69, with no
advisory or exit-status changes. Input roots, output hashes and counts are saved
beside this note. Counts include the deliberately malformed grammar corpus.

Validation passed:

- Canonical locked all-features Rust CI scope: 13,288 library tests (three existing
  ignored), 449 binary tests and 1,389 integration tests. Default threading and
  deadlines were retained, with isolated fixture temporary directories.
- Compiled native API: all 575 tests in Linux CI's 21 files, including async
  CommonJS, ESM and project parity.
- LLVM: regions 98.54%, functions 99.34%, lines 99.32%. No filename exclusions.
  The full PR diff measures 1,177/1,196 executable lines (98.4114%), above 98%.
- `cargo clippy` with warnings denied, Rust formatting, duplicate and spelling checks,
  JavaScript lint/format, import resolution and repository check.
- Complexity: 15,410 functions analyzed; none exceed CRAP 30.

The LCOV report and complete test logs are retained locally under
`/tmp/nm-2682-commands-evidence`. The coverage target grew to 11 GB and was removed
immediately after exporting the reports. Both worker worktrees were removed.
No main/shared target directory was used or removed.

To repeat coverage, use a private target, source `cargo llvm-cov show-env --sh`,
then run `bash .github/scripts/rust-validation.sh test`. Export LCOV with
`cargo llvm-cov report --lcov --output-path /tmp/current.lcov`, and run
`python3 check-patch-coverage.py /tmp/current.lcov --base origin/main` from the
repository root. Global floors are each 98%. The checker uses the merge base and
all changed Rust executable lines; it does not exclude filenames.

`compare.py` records exact output hashes using `fixture-comparisons-input.json`.
Its fixed paths identify the frozen baseline and evidence directory used in
this run. `validate-postgres.py` records the isolated PostgreSQL validation and
always stops its instance and removes its temporary data directory.
