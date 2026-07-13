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

Both rules below are exactly that: "does this Rust function/call have shape
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

Both rules use `severity: error`, so `ast-grep scan` exits non-zero on any
finding — no extra flag needed.

## Rules

### `no-ungated-directory-walk`

Flags a raw `std::fs::read_dir(...)` or `WalkDir::new(...)` call expression
under `crates/no-mistakes/src/**`. Neither has `.gitignore` awareness beyond
whatever directory names get hardcoded into a denylist, so an unguarded
recursive walk can descend into huge generated/vendored directories
(`node_modules`, `.next`, build output) that `git ls-files` would never
surface — see `crates/CLAUDE.md`'s "Never walk the tree without `.gitignore`
awareness" section for the full explanation and the preferred fix order
(derive from the git-visible file list first; fall back to the `ignore`
crate's `WalkBuilder` only outside a git repo).

The rule carries a small, explicit `ignores` allowlist for call sites that are
either the documented non-git fallback itself or are genuinely bounded
(single-level `read_dir`, or looping over a small explicitly-configured
directory list) — see the comment at the top of
`.ast-grep/rules/no-ungated-directory-walk.yml` for the reasoning behind each
entry. Sites are added to the allowlist deliberately and individually, not via
broad glob categories, so a new raw walk anywhere else still gets caught.

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
