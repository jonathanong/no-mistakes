# Code duplication

Run `pnpm run lint:duplicates` from the repository root. The lint job in
`.github/workflows/ci.yml` runs the same command using the locked jscpd version.
Any reported clone fails the command; a scan that analyzes no files also fails.
There is no baseline or overall duplication-percentage allowance.

## Detection

`.jscpd.json` explicitly scans Rust, JavaScript, JSX, TypeScript, and TSX in
`crates/` and `packages/`. It also names the production planner at
`crates/no-mistakes/src/tests/` as a scan root so jscpd's function-similarity
pass treats those files as production code. Overlapping roots still read each
file once. Exact token clones need at least 30 lines and 50 tokens. The `weak`
mode ignores whitespace and comments. JavaScript, JSX, TypeScript, and TSX share
one exact-clone comparison pool; jscpd strips erasable TypeScript syntax when
comparing across that family.

The additional function similarity pass requires at least 30 lines and 20
normalized syntax-tree nodes, with a minimum score of 0.80. This score measures
shared subtrees, not the percentage of identical source text. Normalization
ignores local names, field names, and literals; called function names and
operators remain significant. The token minimum applies to exact detection,
not the function similarity pass. No embedding model or persistent analysis
cache is used.

The settings were selected by scanning this repository and inspecting newly
reported pairs as each minimum was lowered. Smaller blocks and lower scores
began matching unrelated visitor plumbing and lists with different meanings.
Treat a finding as a prompt to inspect semantics before extracting a helper.

## Scope

The scan respects `.gitignore`. Explicit exclusions cover fixture directories,
test files named `tests.rs`, `tests_*.rs`, `*_tests.rs`, `*.test.*`, or `*.spec.*`,
directories named `test`, crate integration-test trees, nested `src/<module>/tests/`
directories, Rust `test_support.rs` files and `test_support/` directories,
dependency directories, and Cargo build output. Tests and fixtures intentionally
repeat setup and examples. The root `src/tests/` planner and directories such as
`integration_tests` remain in scope because they contain production analyzers.
Local scans can include untracked files allowed by `.gitignore`; CI uses a clean checkout.

## Fixing findings

Extract the repeated operation into a shared helper and retain domain-specific
policy at each caller. Preserve source-session ownership, fact demand, resolver
scope, error precedence, and deterministic output. Use the existing fixture
suites to check behavioral parity, then rerun the duplication check.

Do not rename locals, arbitrarily split functions, or add broad exclusions to
hide a finding. If a pair has distinct semantics and cannot share meaningful
logic, document that evidence before introducing any narrow exception.

For a detailed report, run `pnpm exec jscpd --reporters json --output /tmp/no-mistakes-duplicates`.
Reports are diagnostic artifacts and are not committed.
