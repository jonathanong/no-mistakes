# `workflow-topology-policy`

Declarative GitHub Actions topology assertions over the graph produced by
`ciTopology()` / `createWorkflowTopologyIndex()`. Configure inventory,
required and forbidden jobs and `needs` edges, artifact edges, exact
fan-in, reusable-workflow callers, step order, and unlocked-workflow
reasons.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      jobInventory:
        .github/workflows/ci.yml: [lint, test]
      requiredDirectEdges:
        - [".github/workflows/ci.yml#lint", ".github/workflows/ci.yml#test"]
      stepOrders:
        - jobId: ".github/workflows/ci.yml#lint"
          steps:
            - uses: actions/checkout@v4
      unlockedWorkflowReasons:
        .github/workflows/ci.yml: "single-job lint has no overlapping work"
```

`jobInventory`, `exactCallerJobs`, and `unlockedWorkflowReasons` are
checked only when those maps are non-empty so a rule can assert a single
edge without listing every workflow.

Counterexample: `test` does not `needs: lint`, or a required step is
missing.

```yaml
jobs:
  lint:
    runs-on: ubuntu-slim
    steps:
      - run: pnpm lint
  test:
    runs-on: ubuntu-slim
    steps:
      - run: pnpm test
```

## Why and when

Use this rule when CI's job, artifact, reusable-workflow, and step ordering are
part of the delivery contract and should be checked as a graph.

## What it catches/requires

Configured inventory and assertions must match workflow topology: required or
forbidden edges, exact fan-in, artifacts, callers, step order, and documented
unlocked workflows.

## Options and defaults

All collections default to empty, so omitted assertions impose no requirement:

- `jobInventory`: workflow path to the exact expected job IDs.
- `requiredJobs` / `forbiddenJobs`: job IDs that must exist or must not exist.
- `requiredDirectEdges` / `forbiddenDirectEdges`: `[from, to]` pairs for direct
  `needs` edges.
- `requiredTransitiveEdges` / `forbiddenTransitiveEdges`: `[from, to]` pairs
  checked across any dependency path.
- `requiredArtifactEdges`: objects with `from`, `to`, `name`, and optional
  `match` artifact-kind selector.
- `exactFanIns`: job ID to the complete sorted list of direct upstream jobs.
- `exactCallerJobs`, `stepOrders`, and `unlockedWorkflowReasons`: reusable
  caller, ordered-step, and documented-unlocked-workflow policies.

Empty maps do not assert that every possible workflow is listed; each supplied
job or edge is checked and stale required targets are findings.

## Valid example

```yaml
jobs:
  test:
    needs: lint
```

## Counterexample

```yaml
jobs:
  test:
    steps: [{run: pnpm test}]
```

## Fix

Add the missing topology edge or ordered step, or update the policy with the
intended graph and a reason for an intentionally unlocked workflow.

## Suppression

Prefer an `unlockedWorkflowReasons` entry for a deliberate exception. Use a
file directive only when the workflow is owned by an external generator.

## Related rules

[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) checks typecheck gates;
[`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks test path filters.

Fix: add the `needs` edge and ordered steps the policy names, or update
the YAML options to match the intended graph.

```yaml
jobs:
  lint:
    runs-on: ubuntu-slim
    steps:
      - uses: actions/checkout@v4
      - run: pnpm lint
  test:
    needs: lint
    runs-on: ubuntu-slim
    steps:
      - run: pnpm test
```
