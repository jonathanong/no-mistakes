<!-- cspell:ignore jonathanong -->

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
  repeated graph builds, output parsing, subprocess overhead, and reliability issues
  like jonathanong/filaments#4058.
- N-API functions must be asynchronous. CPU-heavy analysis must run through
  libuv async tasks and JS-facing helpers must return promises rather than
  blocking the Node event loop.
- Fully parallel, deterministic output — independent file analysis and domain
  checks should use rayon/concurrent data structures, then sort/merge before
  rendering results.
- No hardcoded domain conventions — route roots, HTTP prefixes, queue factories,
  workers, and similar project-specific locations must come from configuration.
- Full-suite, repository-wide, and global fallback behavior must be explicit
  opt-in. Do not add default-on global behavior or infer project conventions
  when a scoped configuration knob can make the behavior intentional.
- Rules to keep the AST parsable (e.g. no indirection, no dynamism)
- Reduce Agent token usage
- Allow custom error messages for agents
- Automatically fix when possible
- If a rule is file-specific, make it an eslint/oxlint rule
- 100% test coverage
- Test fixture-based — can't be perfect, but add more tests to improve coverage
- Heuristics — can't be perfect, but we'll try our best
- All CLIs must also be available through the N-API API for node.js

## Prepared analysis ownership

- A public CLI, N-API entrypoint, or integration runner creates exactly one
  request-scoped analysis session. It owns the visible-file inventory,
  `SourceStore`, requested TS facts, and canonical relationship data for that
  request. Lower layers borrow those prepared inputs; they do not create a
  competing session, inventory, store, fact pass, or equivalent graph.
- Semantically distinct resolver/catalog projections may coexist inside that
  ownership boundary when one request needs them (for example, ordinary
  codebase resolution plus broader test-runner project resolution). They must
  share the request inventory, sources, and union fact pass, and each output
  field must use the projection whose scope defines its public semantics.
- Declare the complete fact and relationship demand at the boundary before
  collection. Domain checks and graph edge producers consume the prepared
  `TsFactLookup`/fact map, including failure entries, instead of collecting
  domain facts or parsing a second time.
- The session is also the ownership boundary for source text. Every source
  consumer reads through its session-provided `SourceStore`, so a successful
  read and an I/O failure both have one request-wide identity and accounting
  path. A helper that accepts a path but not prepared sources is a design smell
  for any TS/JS source consumer.
- Relationship analyzers emit typed relationships once into the canonical
  dependency graph (or its prepared symbol/reverse projection). Commands,
  checks, and reports project/filter those relationships; they must not each
  rebuild an equivalent reverse index or a private graph shape.
- Treat the resolver catalog, candidate-file universe, configuration fallback,
  and relationship filters as part of a prepared analysis's semantics, not just
  implementation details. Reuse a prepared graph or reverse projection only
  when those inputs are equivalent for the fields being produced; a broader
  runner/project catalog must not answer an ordinary codebase query.
- Additive CLI flags must not change pre-existing report fields. For example,
  requesting test impact may add `testImpact`, but it must not change ordinary
  `directImporters` or `dependentsCount`. When an additive analysis uses broader
  resolver scope, add a fixture-backed parity test that compares the baseline
  fields with the flag both off and on.
- Keep bindings declarative per layer: extract syntactic imports, exports, and
  domain occurrences into facts; resolve paths and ownership in the prepared
  resolver/catalog layer; then project graph/query relationships. Do not blend
  file reads, parsing, resolution, and output-specific traversal in a command
  handler just because it is convenient locally.

## Context Management

- By default, show minimum output
- When showing errors, explain what the error is, where it is, how to fix it. For `check` rules, explain why this check exists.

## Development

- When finding an error, always create a regression test
- Continuously add test fixtures to `fixtures/**` for cases you find
- Test fixtures live under `fixtures/<category>/<name>/` at the repo root. Do NOT create fixtures inline in test code (no `fs::create_dir_all` / `fs::write` to build a fixture during a test run). Save the files to `fixtures/*` and reference them via the per-crate / per-package fixture helper.
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
- Rule suppression must work consistently for every `no-mistakes` rule. Use `no-mistakes` suppression directives, never `guardrails`, and support top-of-file opt-outs (`no-mistakes-disable-file`) plus line-specific opt-outs (`no-mistakes-disable-line` and `no-mistakes-disable-next-line`) where findings have line numbers.
- All shared Rust code belongs in `no-mistakes`. Crates must not depend on one another directly. If two crates need the same helper, lift it into `no-mistakes` first.
- After `cargo build`, `cargo test`, or `cargo bench` in a worktree, delete
  the workspace-root `target/` (`cargo clean` from any crate, or `rm -rf`
  `<workspace-root>/target`). Cargo writes artifacts at the workspace root
  even when invoked from `crates/no-mistakes`. If a local Criterion
  `--save-baseline` / `--baseline` comparison is in progress, wait until that
  comparison finishes — the first run stores measurements under `target/`.
  Do not keep build artifacts in throwaway worktrees afterward. Never delete
  the main workspace `target/` unless this worktree is the main workspace.
- Local Criterion is the source of truth for performance before/after only
  when both runs use the same machine, toolchain, benchmark mode, thread
  count, and fixture. GitHub-hosted runners do not stick to one CPU
  architecture (Intel vs AMD; Macro Runners are ARM64), and CodSpeed may
  compare against an older base; those mismatched comparisons are not proof
  of a win or a regression. Treat a CodSpeed failure as actionable only when
  the report compares the same runtime environment and the expected base
  commit.

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
  or rule page.

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

- Coverage gates must enforce 99% line and function coverage by default.
  Change `RUST_COVERAGE_FAIL_UNDER_LINES` and
  `RUST_COVERAGE_FAIL_UNDER_FUNCTIONS` in `.github/workflows/ci.yml`, and keep
  Codecov rust `target` values in `codecov.yml` in sync.
- **Never** use `cargo llvm-cov --ignore-filename-regex` to suppress uncovered source files. The only files exempt from coverage are test files (`tests/`, sibling `tests.rs`) and test fixtures (`fixtures/`), which `cargo llvm-cov` already excludes by default.
- If a file cannot be brought to the configured gate, refactor it (extract logic to a lib, thin the entry point) — do not add an exception.
