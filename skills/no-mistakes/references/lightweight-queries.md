# Lightweight single-file queries — full reference

Short-form queries for one file that skip structural-question formulation. Reach
for these when the question is local ("what imports this file?", "is this export
used?") and a full `dependents` traversal is more setup than the answer needs.
All emit JSON on non-TTY (or with `--json`) and accept `--root`, `--tsconfig`,
`--format`, and `--json`.

## When to use a different tool

- Need **transitive** impact or non-import edges → `no-mistakes dependents`.
- Need a symbol's **public-API list** without consumers → `no-mistakes symbols`.
- Need exact call **text** or line context → `rg` on the files these return.

## Resolution scope

These commands run one reverse import scan that resolves **relative**,
**tsconfig path**, and recognized **workspace package** imports through visible
package entry points. Configured tsconfig `paths` aliases or `baseUrl` resolution
takes precedence and can resolve a subpath even when the package's `exports`
blocks it. Otherwise, missing or blocked workspace subpaths do not create import
edges. Unmatched third-party packages are outside this project-wide index.
(Relative, alias, NodeNext `.js`, and declaration-only `.d.ts` imports are also
resolved.) When a module ships a runtime `.js` and a `.d.ts` side by
side at the same path, resolution prefers the runtime file, so a type-only
import may attach to the `.js` rather than the `.d.ts`.

## `importers <file>`

Direct importers of one file plus a `dependentsCount`. One reverse import scan,
no full graph build.

```bash
no-mistakes importers src/utils.mts --format json
no-mistakes importers src/utils.mts --tests --format json   # adds testImpact (builds graph)
```

`--tests` adds the transitive impacted-test set under `testImpact`.

## `exports-of <file>`

Named exports and, for each, who imports it.

```bash
no-mistakes exports-of src/components/Tabs.tsx --format json
no-mistakes exports-of src/components/Tabs.tsx --no-importers   # export list only, instant
```

Re-exports include a resolved `source`. Namespace imports (`import * as ns`) are
counted at file granularity, not per-export.

## `dead-exports <file> [NAME...]`

Yes/no on whether anything still imports the given exports (all exports if no
names). Exits non-zero when any are dead. Works before or after deleting the
export. Counts import edges only — dynamic/string-keyed access is not detected.

```bash
no-mistakes dead-exports src/utils.mts --format json
no-mistakes dead-exports src/utils.mts oldHelper legacyFn --format json
```

## `call-sites <file> SYMBOL`

Every call site of an exported function with coarse argument shapes (`string`,
`number`, `object`, `array`, `arrow`, `spread`, `other`, …) — no type inference.
Follows named and star re-export barrels and accounts for renamed local export
bindings. A star barrel that explicitly exports a same-named local symbol can
make its consumers over-report for the target export. Import-then-re-export
chains are not followed: the barrel can be scanned while downstream callers are
omitted. Matches direct identifier calls only (`fn(...)`), not `ns.fn()` or
aliased indirection.

```bash
no-mistakes call-sites src/api.mts handler --format json
```

## `resolve-check <file> [file...]`

Whether every import in the file resolves. Fully local, sub-second. Each import
is `resolved`, `external` (third-party/builtin/unknown package), or `unresolved`
(a broken relative, aliased, or recognized workspace import). Workspace
`exports`/`main` targets agree with dependency `workspace` edges; missing or blocked
subpaths fail closed. Exits non-zero when any are unresolved. Omit `--tsconfig`
for ordinary monorepo ownership; pass it only to force a resolver.

```bash
no-mistakes resolve-check src/new-feature.test.ts --format json
no-mistakes resolve-check src/new-feature.test.ts src/changed-module.ts --format json
```

One input returns the historical per-file result. Multiple inputs return sorted
`results`, an `allResolve` summary, and sorted `unresolvedFiles`; duplicate
paths are checked once. The batch uses one visible-file inventory and one
parallel union import-fact pass.
