# Bounded statements: issue 2704

Baseline: freshly fetched `origin/main` at `17139a03be47301db3164557d609fd2a7d376a68`.
The PR starts from that commit and contains no unmerged predecessor branches.
Binary and production-source hashes are in [validation.json](validation.json).

The stated `DELETE USING` / `UPDATE FROM` example already reports on this baseline
with complete catalog index metadata. Saved controls now lock unbounded joined
sources, independently pinned targets, bounded sources, unique-key chains, and an
unbounded sibling beside a bounded chain. This part adds coverage and explanation;
it does not change the existing target-only mutation proof.

Two production changes reproduce independently:

- A malformed catalog index with a placeholder in its predicate or expression key
  previously proved a lookup when query text matched. Both now remain reported.
- A nullable `NULLS NOT DISTINCT` key queried with `IS NULL` previously reported.
  It now proves a lookup; ordinary nullable UNIQUE remains reported.

The stale expression-index fixture comment is also corrected.

## CLI comparison

Both binaries checked the same 40 saved bounded-statements configurations with
`check --root <configuration-directory> --config <filename> --format json`.
[comparison.json](comparison.json) records the exact finding changes. The other
39 stdout/stderr/status results are byte-identical. Only the new fixture changes:
two malformed-index findings appear and one NULL lookup finding disappears;
joined DML results stay unchanged. Both runs retain their failing exit status.

## Validation

The complete locked all-features library, binary and integration scope ran under
LLVM instrumentation using `.github/scripts/rust-validation.sh test`, CI-default
threading, and a private Cargo target. It passed 13,263 library tests (three
existing ignored), 449 binary tests and 1,388 integration tests. The focused
bounded-statements suite passed all 234 tests. All 575 compiled native API tests
across CI's 21 files passed, including CJS/ESM/project-report parity.

No coverage filenames were excluded. Unchanged 98% global floors pass: regions
98.55%, functions 99.34%, lines 99.34%. [patch-coverage.json](patch-coverage.json)
measures all 39 executable changed Rust lines as covered from the full LCOV report;
test-only source without LCOV records is listed explicitly. Hosted Codecov remains
authoritative. The 11 GB coverage target was removed after exporting reports.

Rust linting, formatting, spelling, JS resolution and the configured repository check
pass. Structural planning has no configured Rust test selection; the full Rust
scope supplies validation. Signature impact cannot inspect the intentional invalid
UTF-8 fixture, and this CommonJS test helper exports no named signature.
Test planning also warns about an existing dynamic resource path in the native
API suite; running all 21 configured native files covers that unresolved selection.
