# `no-mistakes ci topology`

Parse `.github/workflows` into a typed graph — workflows, jobs, and edges for
`needs` control flow, reusable-workflow calls, `workflow_run` subscriptions,
and same-run `actions/upload-artifact` / `actions/download-artifact`
dataflow — with diagnostics for malformed, dangling, cyclic, or
contract-violating definitions.

```sh
no-mistakes ci topology --format json
no-mistakes ci topology --workflow ci.yml --format mermaid
```

## Options

| Flag | Description |
|------|-------------|
| `--workflow` | Restrict output to this workflow (basename, e.g. `ci.yml`, or a path inside `.github/workflows`) plus its transitive local reusable-workflow callees. Repeatable. Defaults to every workflow. |
| `--root` | Project root directory (default: current directory). |
| `--config` | Path to config file. |
| `--format` | Output format: `json` or `mermaid`. |
| `--json` | Shorthand for `--format json`. |

## Exit behavior

If any diagnostic is an error, nothing is written to stdout — each
diagnostic is printed to stderr as `[<code>] <workflow>(<space><job>)?: <message>`
and the command exits `1`. Output is only printed once the (possibly
`--workflow`-filtered) graph is clean.

## Model

- **Workflows** carry their triggers, `jobIds`, resolved `concurrency`
  (`raw` as declared, `effective` with GitHub's documented defaults filled
  in), and — when reusable — the parsed `workflow_call` contract (inputs,
  secrets, outputs).
- **Jobs** carry their `key`, `kind` (`job` or `matrix-template` when a
  `strategy.matrix` is present), condition, resolved concurrency, and steps.
- **Edges** are one of four kinds: `needs` (job control flow), `calls`
  (reusable-workflow calls — `to` is present only for local `./` targets;
  remote calls are opaque), `workflow-run` (resolved `workflow_run`
  subscriptions, matched case-insensitively by workflow `name`), and
  `artifact` (a same-run `upload-artifact` step feeding a `download-artifact`
  step, tagged `match: exact | pattern | all | possible`).
- **Diagnostics** cover dangling `needs`, job/`workflow_run`/reusable-call
  cycles, duplicate step ids, unknown or non-prior step references,
  duplicate workflow names, missing or non-callable local reusable-workflow
  targets, `workflow_call` contract violations (missing/unknown/mistyped
  inputs, missing/unknown secrets, unknown output references), missing or
  ambiguous `workflow_run` sources, `workflow_run` chains deeper than 3
  levels, malformed YAML, and missing/ambiguous same-run artifact producers
  or an over-limit reusable-workflow expansion (see below).

### Same-run artifact dataflow

A step using `actions/upload-artifact@*` / `actions/download-artifact@*` is
parsed into a `WorkflowStep.artifact` declaration; `download-artifact` steps
with a `current-run` source (the default — no `github-token`/`repository`/
`run-id` override) are resolved against every upload reachable within the
same root workflow's run, following local (`./`) reusable-workflow calls
into the caller's run rather than treating them as separate roots. Each
resolution reports one of four match certainties:

- `exact` / `pattern` / `all` — a single producer is guaranteed to run
  before the consumer (via `needs`, unconditionally, with a single matching
  matrix instance).
- `possible` — a producer *could* supply the artifact but isn't guaranteed
  to (no ordering constraint, a conditional job, more than one candidate, or
  a matrix-multiplied upload), or a dynamic (`${{ ... }}`), path-derived
  (`archive: false`), or remote-call producer might supply it.

When no candidate can possibly satisfy an exact name request, the graph
carries a `missing-artifact-producer` diagnostic instead of an edge; when
multiple unordered or matrix-multiplied candidates compete for the same
exact name, it's `ambiguous-artifact-producer` instead. A local
reusable-workflow call DAG that would expand past 4096 same-run job
occurrences is truncated entirely (`artifact-resolution-limit`) rather than
reporting a partial answer. `artifact-ids` downloads and non-`current-run`
sources are never resolved or diagnosed. A job reachable from more than one
root that would resolve differently per invocation reports neither an edge
nor a diagnostic for that download — an answer that isn't true for every
invocation isn't reported as true at all.

## Output (json)

```json
{
  "schemaVersion": 1,
  "workflows": [
    {
      "id": ".github/workflows/ci.yml",
      "path": ".github/workflows/ci.yml",
      "name": "CI",
      "callable": false,
      "triggers": [{ "event": "push" }],
      "jobIds": [".github/workflows/ci.yml#build", ".github/workflows/ci.yml#test"]
    }
  ],
  "jobs": [
    {
      "id": ".github/workflows/ci.yml#test",
      "workflowId": ".github/workflows/ci.yml",
      "key": "test",
      "kind": "job",
      "steps": [{ "index": 0, "kind": "run" }]
    }
  ],
  "edges": [
    { "kind": "needs", "from": ".github/workflows/ci.yml#build", "to": ".github/workflows/ci.yml#test" },
    {
      "kind": "artifact",
      "from": ".github/workflows/ci.yml#build",
      "to": ".github/workflows/ci.yml#test",
      "name": "dist",
      "producerStep": 2,
      "consumerStep": 0,
      "match": "exact"
    }
  ],
  "diagnostics": []
}
```

This is a stable, versioned schema (`schemaVersion: 1`): field names, field
order, and array/diagnostic sort order are part of the contract. Optional
fields (e.g. a job's `name`, a workflow's `concurrency`) are omitted rather
than emitted as `null` when absent.

`mermaid` renders a `flowchart LR` diagram — workflows as subgraphs
containing their jobs, typed edges, and lock nodes for every declared
`concurrency` block (literal groups sharing a name lock together
case-insensitively; a group containing an unresolved `${{ }}` expression
gets its own lock per declaration).

Node API: `ciTopology(options)`. The query index used to answer
"what does this job depend on" / "who calls this workflow" style questions
(`createWorkflowTopologyIndex()`) is documented in
[`docs/node-api.md`](../node-api.md) — it stays JS-only and is rebuilt from
`ciTopology()`'s output rather than crossing the N-API boundary itself.
