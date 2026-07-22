# `no-mistakes tests impact`

Find impacted tests from explicit changed files.

```sh
no-mistakes tests impact src/api.mts --format json
```

Use this when an agent already knows the changed files and wants a structured
test set without parsing a git diff. Impact traversal is file-scoped today;
`file#symbol` inputs are accepted for compatibility but the symbol suffix does
not narrow the result.

Key options: `--root`, `--config`, `--tsconfig`, `--format`, and `--json`.

## Runtime filesystem resources

Impact traversal also follows supported literal filesystem reads, directory
reads, and static `glob`/`fast-glob`/`tinyglobby` calls. A changed tracked
resource therefore selects tests through its importing consumer. JSON reasons
retain `via: ["resource"]` and include optional edge-aligned `via_details`
with `{ "type": "resource", "consumer_file", "call_sites": [{ "call_kind",
"line" }] }`. Path output still contains only test paths.

Computed paths, patterns, and cwd values never create guessed edges. They emit
`dynamic-resource-path`, `dynamic-resource-pattern`, or
`dynamic-resource-cwd` warnings only when the consumer is relevant to the
requested impact path; they do not enable fallback.

## Dynamic imports and `next/dynamic`

Impact traversal follows runtime `import()` boundaries, including
`next/dynamic(() => import('./Foo'))`: changing `Foo` surfaces the tests that
reach it through the dynamic caller, at `medium` confidence with a
`dynamic-import` warning. This covers the dynamic import assigned to or wrapped
by an exported binding — `export const X = dynamic(…)`,
`export default dynamic(…)`, `const X = dynamic(…); export default X`, and
`export default memo(X)`.

Limitations: computed specifiers (non-string-literal `import(...)`) cannot be
resolved statically. A lazy binding reached only through further indirection —
chained through a second private binding (`const W = memo(X); export default W;`)
or referenced as JSX inside an exported component
(`const X = dynamic(…); export function Page() { return <X /> }`) — is pruned by
reachability analysis and may not be traced. Assign or export the
`dynamic(…)` result directly for reliable detection. Dynamic imports of a
workspace/monorepo package (`dynamic(() => import('@app/foo'))`) are surfaced as
`WorkspaceImport` edges, so they appear at high confidence without a
`dynamic-import` warning, and a type-only workspace import in a registry file may
still produce a hint.

## Stub tests and registry hints (opt-in)

Two [`tests.impact`](../configuration/tests.md#testsimpact) config knobs tune the
output:

- `alwaysIncludeTests` surfaces stub/mock tests (e.g. `**/*.mock.test.*`) that a
  suite `exclude` would otherwise drop, so you can update the stub before
  pushing.
- `registries` emits a `registry-hint` warning when the changed file is imported
  by a registry file (e.g. `auth-gated-code-splitting.mts`), reminding you to
  verify the registry entry.

Node API: `testsImpact(options)`. Both knobs are read from `.no-mistakes.yml`, so
the Node API honors them with no extra options.
