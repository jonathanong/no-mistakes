# `finite-set-consistency`

## Why and when

Use this rule when two explicit inventories must remain synchronized without
maintaining a private script or a hand-reviewed diff.

## What it catches

It extracts configured finite string sets and reports missing, stale, empty, or
non-static members according to the selected comparison mode.

## Options

`sets` entries require `name` and `kind` plus that extractor's documented
fields (`file`, `target`, `key`, `pattern`, `property`, or `minSize`).
`comparisons` entries require `left` and `right` and default `mode` to
`equal-set`; supported modes are documented below.

## Valid example

Two configured route inventories containing the same static values pass an
`equal-set` comparison.

## Related rules

[`pnpm-release-age-policy`](pnpm-release-age-policy.md) uses policy-specific
cross-file consistency; [`structured-config-policy`](structured-config-policy.md)
checks individual config shapes.

Compares named finite string sets extracted from source files, structured
config, docs, and file paths.

```yaml
rules:
  - rule: finite-set-consistency
    scope: repository
    options:
      sets:
        - name: routeType
          file: src/routes/types.ts
          kind: ts-string-union
          target: RouteName
        - name: routeFiles
          kind: path-regex-capture
          pattern: "^src/routes/(?<value>[^/]+)\\.ts$"
          minSize: 1
        - name: workspaceExcludes
          file: pnpm-workspace.yaml
          kind: yaml-sequence
          key: minimumReleaseAgeExclude
        - name: dependabotGlobs
          file: .github/dependabot.yml
          kind: yaml-sequence
          key: updates.0.cooldown.exclude
        - name: permanentPackages
          file: .no-mistakes.yml
          kind: yaml-string-selector
          key: rules.[].options.permanentPackages.[].name
        - name: schedulerIds
          file: backend/queues/ai-agents/enqueues/schedules.mts
          kind: ts-call-first-string-argument
          target: ai_agents.upsertJobScheduler
        - name: scheduledJobRegistryIds
          file: backend/api/v1/mq/scheduled-jobs-ai-agents.mts
          kind: ts-const-array-property
          target: AI_AGENTS_SCHEDULED_JOBS
          property: id
      comparisons:
        - left: routeType
          right: routeFiles
        - left: workspaceExcludes
          right: dependabotGlobs
          mode: glob-coverage
        - left: schedulerIds
          right: scheduledJobRegistryIds
```

Supported set kinds are `ts-string-union`, `ts-const-object-keys`,
`ts-const-object-property`, `ts-array-literal`, `ts-const-array-property`,
`ts-call-first-string-argument`, `yaml-sequence`,
`yaml-string-selector`, `markdown-table-code-cells`, `sql-enum`, and
`path-regex-capture`.

`yaml-string-selector` collects terminal YAML strings at a dot-separated
selector. Bare mapping-key segments select keys, numeric segments select a
sequence index, and `[]` traverses every sequence member. For example,
`rules.[].options.permanentPackages.[].name` collects every package name from
every matching rule option. Use a bracketed JSON string segment for a literal
mapping key that would otherwise be structural: `["a.b"]` selects a key with a
dot, `["[]"]` selects a key named `[]`, and `["0"]` selects a numeric-string
key. JSON escapes work inside bracketed segments, such as `["a\\\"b"]`.

A missing selector, invalid selector syntax, invalid traversal, or non-string
terminal value extracts no values. Pair required selectors with `minSize: 1`
so a renamed YAML path fails closed.

`markdown-table-code-cells` collects inline-code values from table body cells
only; inline code in table headers is descriptive metadata rather than a set
member.

`path-regex-capture` matches visible lexical path entries from the request
inventory: regular files, file-target symlinks, directory-target symlinks
(including hidden layouts such as `.claude/skills/<name>` →
`.agents/skills/<name>/`), and broken tracked or visible links. It compares
the git/discovery path string and does not follow the link or require a
regular-file target. Graph and source discovery still use readable file
targets only, so those views stay unchanged.

Set `minSize` on any set when an empty extract must fail closed. `equal-set`
treats two empty sets as equal, so a renamed pattern or missing directory
would otherwise pass. When an extract has fewer members than `minSize`, the
rule reports that set and skips comparisons that use it. `minSize` defaults
to `0` and is kind-agnostic. Use `minSize: 1` for live inventories such as
skill-directory captures.

`ts-call-first-string-argument` extracts the first argument from every matching
call in the configured file. The argument must be a quoted string or an
expression-free template literal. `target` is an exact syntactic callee name,
such as `ai_agents.upsertJobScheduler`; identifier and one-level static member
calls are supported. Calls on local or parameter receivers are included, while
function and method declarations are not calls and are excluded. Calls through
aliases, computed or optional properties, multi-hop member expressions,
interpolated templates, and other dynamic expressions are not inferred.

This extractor is fail-closed: a configured target with no matching calls, or
a matching call whose first argument is not a static string, produces a
finding. This prevents a renamed API or dynamic scheduler ID from silently
making the extracted set empty. Duplicate call values are treated as one set
member, as with the other finite-set extractors.

Comparison modes:

- `equal-set` is the default and requires both sets to contain the same values.
- `glob-coverage` requires every left value to be matched by at least one glob
  string from the right set.
- `mention` requires every left value to appear in the right extracted mention
  set, such as markdown table code cells.

Counterexample: a TypeScript union includes `"settings"` but no matching route
file exists, a workspace YAML allowlist names a package missing from a TS
registry, a registry package is not covered by any dependabot glob, a package
is missing from a markdown policy table, a scheduler registration is missing
from its registry, or two `path-regex-capture` sets both extract nothing and
would pass `equal-set` without `minSize: 1`. Likewise, a
`yaml-string-selector` that no longer reaches `permanentPackages` silently
extracts no values unless it has `minSize: 1`.

Fix: add the missing value to the other set, remove stale values, replace a
dynamic call argument with a static string when it is part of the checked
finite set, restore the files or pattern that should populate a `minSize`
set, restore the YAML selector path and its `minSize: 1`, or narrow the
configured extraction.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for finite set mismatches.
