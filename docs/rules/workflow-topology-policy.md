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
