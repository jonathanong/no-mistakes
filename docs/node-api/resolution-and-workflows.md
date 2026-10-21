# Resolution and workflow APIs

`compilerOptions.paths` uses TypeScript's selection rules in standalone graph
APIs and `analyzeProject()`: exact keys win, then the longest prefix before a
wildcard, with declaration order breaking ties. Replacement fallbacks are tried
only within the winning key, so a missing specific target cannot resolve through
a less-specific mapping.

`ciTopology(options)` returns the same schema-v1 `WorkflowTopology` JSON as
`ci topology --format json` — it never throws on diagnostics (unlike the CLI,
which exits non-zero and prints nothing when any diagnostic is an error);
callers inspect the returned `diagnostics` array themselves.
`createWorkflowTopologyIndex(topology)` builds a frozen, sorted query index
(`directUpstreamJobIds`, `transitiveCalleeWorkflowPaths`,
`artifactConsumersForProducerJob`, etc.) over that result — it is pure JS,
runs entirely client-side, and never crosses the N-API boundary itself:

Workflow, job, and step nodes also expose authored CI configuration:
environment-variable blocks, static secret-reference names, job runner and
timeout declarations, effective permissions, job outputs, and step
`run`/`with` data. Values are not evaluated. Secret analysis is strictly
name-only and never reads GitHub or process secret material. See
[`ci topology`](../cli/ci-topology.md) for the exact schema and normalization
rules.

```js
const { ciTopology, createWorkflowTopologyIndex } = require("no-mistakes");

const topology = await ciTopology({ root: process.cwd() });
const index = createWorkflowTopologyIndex(topology);
index.transitiveDownstreamJobIds(".github/workflows/ci.yml#build");
```

`impactedChecks(options)` shares one in-memory analysis pass across configured
test frameworks. Pass `timings: true` to include an ordered `timings` array in
the report:

```js
const { impactedChecks } = require("no-mistakes");

const report = await impactedChecks({
  root: process.cwd(),
  changedFiles: ["src/api.mts"],
  genericOnly: true,
  timings: true,
});

// report.timings: [{ phase: "prepare", duration_ms: 12 }, ...]
```

Timing entries use stable phase identifiers and fractional-millisecond
durations. The lazy `graph` phase is present only when dependency analysis is
needed. The property is omitted by default. Unlike CLI `--timings`, Node timing
collection does not print progress to stderr.

If no checks are selected, the report includes `empty_result` with a stable
`code` (`no-changed-files` or `no-impacted-checks`) and a human-readable
`message`. It is omitted from reports containing checks. The async API remains
stderr-free, including for empty results.

Set `genericOnly: true` to return only configured `checks.commands` entries.
It preserves changed-file collection but skips test-framework discovery and
selection; its report has no warnings or full-suite fallback, and timed calls
report `prepare`, `generic-checks`, then `total`.

[Node/N-API guide](../node-api.md)
