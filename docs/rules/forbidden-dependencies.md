# `forbidden-dependencies`

Prevents configured source roots from importing forbidden files or modules.

## Why and when

Use this rule when a code boundary must remain one-way, such as client code
that must not reach server-only modules through an indirect graph path.

## What it catches

It traces the configured relationship families from each selected root and
reports a path that reaches a forbidden file or module.

## Options

`roots` selects literal files, directories, or glob patterns as boundary origins.
`excludeRoots` removes origins using the same selectors. `forbiddenModules` and
`forbiddenFiles` are glob patterns for blocked module specifiers and file paths.
`relationships` selects graph edge kinds. Omitted `relationships` follows all
standard edge families; other lists default empty.

Directory roots select source files recursively from the request's discovered
graph, within the rule's project, include, and exclude filters. File roots remain
valid, missing roots are skipped, and overlapping roots select each source once.
Each finding names its source file and includes a query for that file.
Reachability may follow dependencies outside the origin directory.

```yaml
rules:
  - rule: forbidden-dependencies
    projects: [web]
    options:
      roots: ["web/app"]
      forbiddenModules: ["fs", "node:*"]
      relationships: [import, workspace]
```

### Glob semantics

File globs match paths relative to the invocation root, using `/` separators.
They support `*`, `**`, `?`, character classes such as `[ab]`, and brace
alternatives such as `{ts,tsx,mts}`. Matching is case-sensitive; `*` can span
path separators, as in the existing `forbiddenFiles` matcher. Prefer `**` when
expressing recursive directory policies explicitly. Patterns do not resolve
import specifiers: the shared resolver resolves imports first, and the rule
matches their canonical graph file paths.

Root patterns expand only over the request's discovered graph files; they do
not walk the filesystem or widen the configured project/include scope. A valid
pattern with no matches selects nothing. Literal file and directory roots,
including absolute paths, retain their existing behavior. Overlapping selectors
select each origin once. Use `excludeRoots` or the rule's top-level `exclude`
to remove test files as origins; excluding an origin does not remove that file
as a reachable forbidden target. Use explicit exclusions rather than `!` patterns.
Malformed glob patterns report configuration findings at their option entries.
An existing literal file or directory takes precedence over glob interpretation,
so a literal root containing brackets remains usable.

### Per-root exceptions

`exceptions` is a list of `{ roots, allowedFiles }` objects, defaulting to `[]`.
Its `roots` uses the same origin selectors. Its `allowedFiles` uses the same
root-relative target globs as `forbiddenFiles`. Matching exceptions allow only
those file targets for that particular origin; their allowed target sets are
combined when several entries match. They do not allow forbidden modules and
do not change other origins. Traversal continues through allowed files, so an
allowed helper that reaches another forbidden file still produces a finding.

For example, keep production code from reaching tests or stories, while allowing
one coverage script to reach a particular helper directory:

```yaml
rules:
  - rule: forbidden-dependencies
    scope: repository
    options:
      roots:
        - "backend/{api,services,workers}/**"
        - "web/**"
        - "ci/check-*.mts"
      excludeRoots:
        - "**/*.{test,spec}.{ts,tsx,mts}"
        - "**/vitest.setup*.{ts,mts}"
        - "**/test-helpers/**"
        - "**/__tests__/**"
        - "**/__fixtures__/**"
        - "**/*.stories.*"
        - ".storybook/**"
      forbiddenFiles:
        - "**/*.{test,spec}.{ts,tsx,mts}"
        - "**/vitest.setup*.{ts,mts}"
        - "**/test-helpers/**"
        - "**/__tests__/**"
        - "**/__fixtures__/**"
        - "**/*.stories.*"
        - ".storybook/**"
      exceptions:
        - roots: ["ci/check-coverage-scope.mts"]
          allowedFiles: ["test-helpers/vitest-config/**"]
      relationships: [import, workspace]
```

A production file importing a shared utility that imports a test helper fails
transitively. A test importing production code does not fail this policy when
the test is excluded as an origin. The script exception permits only matching
helper targets; it cannot reach other test helpers or stories.

## Valid example

The public-boundary import in the compliant example passes because its selected
relationships cannot reach the server-only implementation.

```ts
// web/app/page.ts
export const page = true;
```


## Counterexample

```ts
// web/app/page.ts
import 'fs';
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges dependency edges in the canonical graph, where unresolved specifiers are already explicit graph facts, so `options.unanalyzable` has no effect on it.

## Related rules

[`forbidden-workspace-closure`](forbidden-workspace-closure.md) checks package
manifest closure rather than source graph edges; [`server-route-client-boundary`](server-route-client-boundary.md)
is a narrower server-route policy.

Counterexample: client code importing a server-only package.

Compliant example: client code imports from `web/app/public-api.ts`, and that
public boundary owns any server-only implementation detail.

Fix: move the dependency behind an allowed boundary or remove the import.

`relationships` limits which graph edges participate in reachability. Use
`import` plus `workspace` for import/module boundaries: `import` includes
static, dynamic, and `require()` imports as well as compile-time type-only
dependencies, while `workspace` follows imports through local workspace package
entry points. Omitting `relationships` uses every standard relationship family,
including routes, tests, queues, resources, and Playwright edges; keep that
broader behavior only when the boundary intentionally spans those domains.

Suppression caveat: suppress only with a `no-mistakes` directive and a concrete
justification, and prefer narrowing `forbiddenModules` or configured roots when
the boundary is intentionally allowed. Review suppressions during boundary
changes.
Findings attach to the selected origin at line 1. For example, place
`// no-mistakes-disable-file forbidden-dependencies -- temporary boundary migration`
in that origin file. A directive in the forbidden target does not suppress a
finding attached to a production origin.
