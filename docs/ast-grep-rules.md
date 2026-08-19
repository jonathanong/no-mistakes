# ast-grep Regression Rules

This repo uses [`ast-grep`](https://ast-grep.github.io) for pure structural
(AST-shape) checks over this crate's own Rust source, in addition to
`no-mistakes check` (which answers graph-aware and config-aware questions
about the *codebases `no-mistakes` analyzes*, not about `no-mistakes`'s own
source).

## Why `ast-grep`, not a native `no-mistakes check` rule

`skills/no-mistakes/references/limits-and-fallbacks.md` draws the line:

> `no-mistakes` answers **graph-aware** and **config-aware** queries. It does
> not ship a structural pattern matcher. For a pure structural blast-radius
> question — matching an AST shape regardless of the import graph — reach for
> `ast-grep` directly.

The rules below are exactly that: "does this Rust function/call have shape
X?", with no dependency-graph or project-config awareness required. That
places them on the `ast-grep` side of the tool boundary, run at pre-push/CI
time, rather than as a new native Rust-source `no-mistakes` rule.

## Project layout

- `sgconfig.yml` (repo root) — points `ruleDirs` at `.ast-grep/rules` and
  `testConfigs` at `.ast-grep/rule-tests`.
- `.ast-grep/rules/*.yml` — one rule per file.
- `.ast-grep/rule-tests/*.yml` — `valid`/`invalid` snippets per rule, run via
  `ast-grep test --skip-snapshot-tests` (no `fix:` is used by either rule, so
  there is nothing to snapshot).

Run locally:

```sh
ast-grep scan                                    # scan the whole project
ast-grep test --skip-snapshot-tests               # run rule test cases
```

All rules use `severity: error`, so `ast-grep scan` exits non-zero on any
finding — no extra flag needed.

## Rules

### `no-ungated-directory-walk`

Flags raw `std::fs::read_dir(...)`, `fs::read_dir(...)`, `WalkDir::new(...)`,
or `walkdir::WalkDir::new(...)` call expressions
under `crates/no-mistakes/src/**`. Neither has `.gitignore` awareness beyond
whatever directory names get hardcoded into a denylist, so an unguarded
recursive walk can descend into huge generated/vendored directories
(`node_modules`, `.next`, build output) that `git ls-files` would never
surface — see `crates/CLAUDE.md`'s "Never walk the tree without `.gitignore`
awareness" section for the full explanation and the preferred fix order
(derive from the git-visible file list first; fall back to the `ignore`
crate's `WalkBuilder` only outside a git repo).

Production discovery has no file-level exemptions. Non-git fallbacks must use
the `ignore` crate, and bounded discovery must still derive from the shared
git-visible candidate list so later edits cannot introduce an unchecked
recursive walk inside an exempt file. The rule's sole `ignores` entry is a
test-only helper that lists one small checked-in parser fixture directory.

### `no-cache-disabling-builder`

Flags a consuming builder method (`fn foo(mut self, ...) -> Self`) whose body
assigns `false` to a field whose name looks cache-related (matched via a
`regex` constraint on the captured field, e.g. `cache_enabled`), as an
undocumented side effect of configuring something unrelated. See
`crates/CLAUDE.md`'s "Verify a builder method doesn't silently disable an
existing cache" section — this is the exact shape of the historical
`ImportResolver::with_visible` bug, where setting a visible-file set also
silently set `cache_enabled = false`.

Methods named `without_*`, `no_*`, or `disable_*` are exempted (a `NAME`
constraint using `not: { regex: "^(without_|no_|disable_)" }`) — those names
signal an intentional, explicitly-named opt-out (e.g. `without_cache()`) and
should stay clean.

Known limitation: this only catches a `false` literal assigned directly to a
matching field inside the builder's own body. It will not catch the same side
effect performed indirectly (a helper method call, `std::mem::replace`,
`Option::take`, etc.).

### `no-direct-playwright-route-scan`

Flags a direct call to `crate::routes::collect_routes(...)` from
`codebase/dependencies/graph/**` — the `DepGraph` edge-producer directory.
`get_or_compute_playwright_routes` (`graph/fact_lookup.rs`) is a *no-key*
shared cache: every caller within one `no-mistakes check` invocation wants
the exact same app-wide Playwright route scan, so there is never a
legitimate reason for an edge producer to call `collect_routes` directly
instead of going through the shared cache. See `crates/CLAUDE.md`'s "Edge
producer smell" note — this is the exact shape of the historical
`collect_playwright_route_edges` bug, which independently re-ran the entire
app-wide route scan the `playwright` rule's own check pipeline already
shares.

`edge_playwright_routes.rs` is exempted: it's the one producer that
legitimately calls `collect_routes` at all, inside the `compute_routes`
closure passed to `facts.get_or_compute_playwright_routes` (with a
`None`-facts fallback calling the same closure directly). ast-grep's
structural matching can't express "this call must be inside a closure passed
to `get_or_compute_playwright_routes`", so the whole file is allowlisted —
narrower than the other two rules' allowlists, which exempt specific call
sites rather than a whole file. A second, unguarded call added anywhere else
in that file would not be caught by this rule; review that file's diffs by
hand for this specific regression class.

### One-pass gateway guards

Four narrow rules protect the shared analysis boundary:

- `no-direct-source-read` covers the TS/JS shared-fact directories, all
  `codebase/queries` source consumers, the demand-driven import traversal,
  shared traversal fact seeder, and prepared integration-runner config readers,
  and requires the canonical `SourceStore::read_path` gateway.
  It deliberately does not flag Markdown, YAML, lockfile, or other document
  readers.
- `no-direct-oxc-parser` blocks new OXC parser construction outside the AST
  gateway. Its exact file allowlist contains legacy source-string extractors;
  remove an entry when its adapter is migrated.
- `no-aggregate-standalone-analysis` covers aggregate `check`,
  `analyzeProject`, and impacted-check aggregation paths. It rejects standalone
  discovery, resolver, graph, fact, or extractor calls that bypass prepared
  invocation state.
- `no-direct-analysis-clock` requires analysis timings to flow through the
  optional invocation observer. Only that observer, compatibility timing
  adapters, and the invocation control-clock adapter used to enforce deadlines
  may construct `Instant`s directly.

These rules use exact path allowlists. They are intentionally narrower than a
blanket ban on reads or parser libraries because non-source documents and
bounded string-only adapters are valid inputs.

### Prepared-analysis regression guards

The following path-scoped rules protect performance and DRY boundaries that
generic Rust/JS patterns can observe:

- `no-unprepared-domain-fact-collection` reserves
  `collect_domain_facts` for the canonical TS fact gateway (including its
  per-file extraction submodule) and the unified check-fact pass. Both own an
  explicit graph fact plan and collect from the same per-file program; every
  leaf consumer must use their prepared facts.
- `no-reverse-query-standalone-index` covers `codebase/queries/**`. Reverse
  queries may not rediscover graph files, including in the projection owner.
  Its companion `no-reverse-query-leaf-projection` prevents query leaves from
  collecting TS facts or building `SymbolIndex`; those operations belong only
  to the prepared build owner in the `reverse` module. Prepared inputs are
  reusable only when their resolver catalog, candidate files, fallbacks, and
  relationship filters preserve the query's semantics; the structural rule
  cannot prove that equivalence, so `CLAUDE.md` additionally requires
  baseline-field parity tests
  when an additive flag introduces a broader analysis scope.
- `no-global-edge-vector-dedup` protects canonical graph finalization in
  `edge_index/build.rs`. A full `edges` vector there must be produced by the
  normalized-adjacency flatten, not globally sorted and deduplicated after the
  fact. Deduplication belongs to per-source adjacency normalization or
  `EdgeIndex`'s keyed batch insertion.
- `no-temporary-arc-symbol-facts` protects shared `FileSymbols` ownership.
  `TsFileFacts.symbols` is `Arc<FileSymbols>`; assigning a deep-cloned inner
  table or wrapping `ts.symbols.clone()` in a new `Arc` duplicates export and
  import tables across check and graph facts. Clone the `Arc` instead. The
  standalone collector may wrap extraction in `Arc::new` once.
- `no-handwritten-json-napi-wrapper` protects the stable Rust N-API
  registration surface, including the query and infrastructure modules.
  Direct `AsyncTask::new(JsonTask::new(...))` functions must be registered via
  `json_binding!` so the libuv/task/name policy has one implementation.
- `no-handwritten-js-json-wrapper` protects the two Node facades. A direct
  static or computed `native.*` JSON parse/stringify wrapper must use the
  shared `callJson`/`createJsonApis` descriptor path instead.

Prepared-symbol catalog reuse is protected primarily by fixture-backed
observer tests because its ownership is semantic rather than a single stable
constructor shape. The Rust/JS wrapper rules above only catch their exact
direct boilerplate; parity tests still cover alias, re-export, and
descriptor-resolution semantics.

### `no-process-spawn-in-file-loop`

Flags `Command::new` inside `for`, `while`, or `loop` bodies in production
source. Process startup must be batched once per invocation or replaced with
an in-process parser/resolver. A canonical one-shot Git discovery subprocess
outside a loop remains valid.

## Adding more rules of this shape

1. `ast-grep new rule -y -l rust <id>` from the repo root creates a stub in
   `.ast-grep/rules/` (and a stub test in `.ast-grep/rule-tests/`).
2. Iterate against real code with `ast-grep scan --rule .ast-grep/rules/<id>.yml <path>`
   before wiring it into `files`/`ignores`/`sgconfig.yml`. Pattern matching in
   ast-grep is structural, not fuzzy: a `pattern:` combined with a `kind:`
   generally needs to include tokens like `pub`/`mut` explicitly if the target
   has them, or you'll get silent zero matches — verify empirically, don't
   guess.
3. Add `valid`/`invalid` cases to `.ast-grep/rule-tests/<id>-test.yml` and
   confirm with `ast-grep test --skip-snapshot-tests`.
4. Only allowlist a file/call site via `ignores` when it's genuinely bounded
   or is the documented fallback itself — leave everything else caught.
5. `ast-grep scan` is already wired into `.husky/pre-push` (alongside
   `no-mistakes check`) and into the `ast-analysis` job in
   `.github/workflows/ci.yml` — a new rule under `.ast-grep/rules/` is picked
   up automatically, no additional wiring needed.
