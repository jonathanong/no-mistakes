## Design principles

Goal: AI-powered AST-based codebase intelligence for AI Agents.

- Determinism + Heuristics over Probabilistic AI
- CPUs are cheaper than GPUs
- Local over Remote
- Stateless across runs — no filesystem caches, databases, or services. In-memory
  per-run memoization and shared fact maps are allowed to avoid duplicate work.
- One pass per invocation — discover files once, parse TS/JS once for the
  requested facts, then reuse those facts across graph construction and checks.
- Cache only in memory — parsed facts, resolver lookups, traversal results, and
  forward/reverse dependency maps may be cached during a run, but never persisted.
- Build one canonical graph — relationship features should produce typed edges
  in the shared dependency graph instead of maintaining separate graph shapes.
- Programmatic API parity — every stable `no-mistakes` CLI capability must expose
  an equivalent N-API interface for Node/programmatic usage. Downstream tools should call
  structured in-process APIs instead of shelling out to `no-mistakes`, avoiding
  repeated graph builds, output parsing, subprocess overhead, and flaky runs.
- N-API functions must be asynchronous. CPU-heavy analysis must run through
  libuv async tasks and JS-facing helpers must return promises rather than
  blocking the Node event loop.
- Fully parallel, deterministic output — independent file analysis and domain
  checks should use rayon/concurrent data structures, then sort/merge before
  rendering results.
- Narrow by configuration: analyze only configured locations (project roots,
  `include`) and targets (executor imports, factories, tags), not every file,
  call, or string that looks like one. Declare a scope once and share it. No
  hardcoded conventions; repo-wide or full-suite fallback is explicit opt-in.
  See [narrowing](docs/why.md#narrowing).
- Change the code, not the analyzer: rules ban hard-to-parse code (indirection,
  dynamism, runtime mutation) and name a static rewrite. Rules judge only what
  they can soundly judge; see [why](docs/why.md).
- Reduce Agent token usage
- Allow custom error messages for agents
- Automatically fix when possible
- If a rule is file-specific, make it an eslint/oxlint rule

## Prepared analysis ownership

One request-scoped session per CLI/N-API/runner call owns the file inventory,
`SourceStore`, declared facts, resolver projections, and canonical graph. Lower
layers borrow them and never re-read, re-parse, re-resolve, or build a private
graph. Details:
[crates/AGENTS.md](crates/AGENTS.md#prepared-analysis-source-session-ownership-and-relationship-projection).

- Distinct resolver/catalog projections (e.g. codebase vs test-runner
  resolution) may coexist but share the inventory, sources, and fact pass. Each
  output field uses the projection that defines its semantics; catalog, file
  universe, config fallback, and filters are part of that semantics, so a
  broader runner/project catalog never answers an ordinary codebase query.
- Additive CLI flags must not change existing report fields (`testImpact` must
  not change `directImporters`). When one widens resolver scope, add a fixture
  parity test with the flag off and on.

## Context Management

- By default, show minimum output
- When showing errors, explain what the error is, where it is, how to fix it. For `check` rules, explain why this check exists.

## Development

- Keep Rust source files within 200 physical lines and test files within 500.
  Audit with `wc -l`; `rust-max-lines-per-file` counts code lines instead, so a
  passing check does not establish this guideline. See the [rule](docs/rules/rust-max-lines-per-file.md).

- Run `rustup update stable` before validation (`rust-toolchain.toml` pins stable).
- When finding an error, always create a regression test
- Continuously add test fixtures for cases you find. Which tree (`test-cases/` or `fixtures/`) a suite loads is in [Contributing](CONTRIBUTING.md); add to the tree its loader reads.
- Do NOT create fixtures inline in test code (`fs::write`, `std::fs::write`,
  `create_dir_all`, `File::create`, or `OpenOptions` `.create`). Save the files
  under `fixtures/` or `test-cases/` and load them through the per-crate fixture
  helper or a `crates/no-mistakes/src/test_support` copy helper such as
  `materialize_saved_fixture`. `no-inline-test-fixture-write` and
  `no-inline-test-fixture-write-cfg` enforce this for `tests.rs`, `*_tests.rs`,
  `tests/**`, `include!` slices of those files, child `mod`s of those files,
  and `#[cfg(test)]` modules
  ([ast-grep rules](docs/ast-grep-rules.md)). Runtime exceptions need a one-line
  justification in `.ast-grep/inline-fixture-write-allowlist.txt`. Existing
  violations are listed in `.ast-grep/inline-fixture-write-baseline.txt`; that
  list can only shrink. Do not add fixture content writes to an allowlisted file.
- Rule output tests commit file snapshots and review those diffs like code.
  Rust `check --format json` goes through
  `crates/no-mistakes/tests/common/check_json_snapshot.rs`. The helper runs a
  saved fixture at `--jobs 1` and `--jobs 2`, then stores one normalized
  report. Normalization rewrites absolute paths to `<root>`, temporary
  directories to `<temp>`, process ids to `<pid>`, and timings to `<timing>`,
  uses `/` separators, and sorts arrays so Linux, macOS, and Windows agree.
  CI sets `INSTA_UPDATE=no` (quoted in `.github/workflows/ci.yml` and
  `release.yml`); a missing or changed `.snap` fails. Regenerate only on
  purpose with `INSTA_UPDATE=1 cargo test -p no-mistakes --test suite_cli_rules check_json_snapshot`
  or `cargo insta review`, then commit the `.snap`.
  ESLint plugin fixture diagnostics use Vitest `toMatchFileSnapshot` with
  fixture-relative `/` paths and sorted findings. CI runs `vitest run` and
  does not pass `-u`. Update with `pnpm exec vitest run <test> -u` only when
  the new output is intended. `agents-md-max-size`,
  `async-call-disposition` proven-consumers, the `test-*` /
  `playwright-*` ESLint diagnostics, and the remaining ESLint rules
  (`packages/eslint-plugin-no-mistakes/test/diagnostic-snapshot.mjs`,
  `snapshotSuite`) are the examples.
- After broad mechanical renames, run `rg` for the old names before the first
  compile to catch stragglers.
- After editing nearby test arguments or fixture paths, re-read the exact diff
  hunk before testing to catch accidental changes to adjacent cases.
- When replacing an analyzer or graph pipeline, compare old and new behavior
  explicitly for public output shape, config fallback, rewrites, and route or
  selector index construction before opening a PR.
- Add short comments to intentionally counterintuitive fixtures or tests so
  reviewers and bots do not "simplify" away the invariant being protected.
- In coverage-gated code, run focused coverage early after adding defensive
  branches; prefer small helpers or refactors when they avoid hard-to-cover
  branches.
- When rejecting an automated review suggestion, add the rationale to the PR's
  Shepherd Journal before resolving the thread.
- Every `no-mistakes` rule supports `no-mistakes-disable-file` (never
  `guardrails`), plus
  `-disable-line` / `-disable-next-line` where findings have lines.
- After cargo builds in a throwaway worktree, `rm -rf <workspace-root>/target`
  (not `cargo clean`, which may follow `CARGO_TARGET_DIR`), after any Criterion
  baseline comparison finishes. Never delete the main workspace `target/`.
- Performance measurement rules and gates:
  [core performance guidance](crates/no-mistakes/AGENTS.md).

### Public surface checklist

A feature is not done until the matching rows land in the same change. Do not
land a CLI or N-API capability without the binding, types, docs, and fixtures
in that PR.

- **Rust library** — the entrypoint, rule, or planner that implements the
  behavior.
- **N-API** — async binding (`json_binding!`), JS facade, and
  `packages/no-mistakes/*.d.ts`. New or widened unions, option fields, and
  report types that callers need to name must be `export`ed. Do not leave a
  public contract only as `SomeType['field']`.
- **Docs (required, not optional)** — `docs/cli/*` for commands,
  `docs/node-api.md` (CLI mapping and runtime export table) for any
  programmatic API, `docs/rules/*` for check rules (example, counterexample,
  fix, suppression, new options), `docs/eslint-rules/*` for lint rules,
  `docs/graph-edges.md` for new edge kinds or filters, `docs/configuration/*`
  for new config knobs, `docs/feature-parity.md` when language or framework
  surface changes, and `skills/no-mistakes` references for agent-facing
  commands.
- **Tests** — fixture-backed behavior, plus declaration locks in
  `packages/no-mistakes/scripts/api.test.js` when `.d.ts` contracts change,
  plus `crates/no-mistakes/tests/docs_coverage.rs` when adding a CLI command
  or rule page. Update and run
  `crates/no-mistakes/src/main/diagnostics_args_tests.rs` for new CLI leaves.
- **Rule snapshots** — new or changed rule output includes a committed `.snap`
  (`*.snap text eol=lf`). Rust `check --format json` goes through
  `check_json_snapshot` for a saved fixture. ESLint plugin fixture diagnostics
  use Vitest `toMatchFileSnapshot`. Review that diff like code. Regenerate only
  with `INSTA_UPDATE=1` or `vitest run -u`.

## Agent Best Practices

### Before editing

- Use `no-mistakes` to scope structural impact before changing an existing
  file: `dependents`, `symbols --mode signature-impact`, and the appropriate
  `tests plan <framework> --changed-file <file>`. For a new file, repeat after
  creating it.
- Pass `--root` explicitly. Omit `--tsconfig` for ordinary monorepo ownership
  resolution; pass a package tsconfig only to force one resolver.
- Use `rg` after graph commands identify files when exact call lines, comments,
  or dynamic syntax matter.

### After editing

- Re-run focused impact/planning for changed files, then ask
  `impacted-checks <file...> --format json` for configured tests and generic
  validation. Inspect warnings and fallback fields before treating an empty
  result as complete.
- Prefer `--format json` for parseable output and `--format paths` only for
  trusted path lists. Do not parse human output or execute untrusted command
  text.

### Before handoff

- Run `resolve-check` for changed TS/JS files and `check --format json` for
  configured repository checks. Also run `playwright check --json` for
  Next.js route, selector, or Playwright changes.
- For repeated in-process queries, use the async Node/N-API API—especially
  `analyzeProject()`—to share one request-scoped analysis rather than spawning
  repeated CLI processes.

## Coverage

- CI enforces the coverage floors (`RUST_COVERAGE_FAIL_UNDER_*` in `ci.yml`).
- **Never** use `cargo llvm-cov --ignore-filename-regex`. If a file cannot meet
  the gate, refactor it (extract logic, thin the entry point); no exceptions.
