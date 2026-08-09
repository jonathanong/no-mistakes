# `finite-set-consistency`

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
        - name: workspaceExcludes
          file: pnpm-workspace.yaml
          kind: yaml-sequence
          key: minimumReleaseAgeExclude
        - name: dependabotGlobs
          file: .github/dependabot.yml
          kind: yaml-sequence
          key: updates.0.cooldown.exclude
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
`markdown-table-code-cells`, `sql-enum`, and `path-regex-capture`.

`ts-call-first-string-argument` extracts the first argument from every matching
call in the configured file. The argument must be a quoted string or an
expression-free template literal. `target` is an exact syntactic callee name,
such as `ai_agents.upsertJobScheduler`; identifier and one-level static member
calls are supported. Calls through aliases, computed or optional properties,
multi-hop member expressions, interpolated templates, and other dynamic
expressions are not inferred.

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
is missing from a markdown policy table, or a scheduler registration is missing
from its registry.

Fix: add the missing value to the other set, remove stale values, replace a
dynamic call argument with a static string when it is part of the checked
finite set, or narrow the configured extraction.

Suppression: use `no-mistakes` suppression directives. Findings currently report
line 1 for finite set mismatches.
