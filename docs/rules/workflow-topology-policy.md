# `workflow-topology-policy`

Declarative GitHub Actions topology assertions over the graph produced by
`ciTopology()` / `createWorkflowTopologyIndex()`. Configure inventory,
required and forbidden jobs and `needs` edges, artifact edges, exact
fan-in, reusable-workflow callers, step order, unlocked-workflow
reasons, secret-name declarations, the intended behavior of each
`concurrency:` block, and shared concurrency groups.

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
      secretInventory:
        repository: [DEPLOY_TOKEN]
        environments:
          production: [RELEASE_KEY]
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
forbidden edges, exact fan-in, artifacts, callers, step order, declared secret
names, documented unlocked workflows, concurrency intent, and shared groups.

## Options and defaults

Topology assertion collections default to empty, so omitted assertions impose
no requirement. `secretInventory` is separately opt-in:

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
- `secretInventory`: optional repository and environment secret-name
  declarations. Organization secrets and Dependabot secrets belong under
  `repository`, not under an environment. Omit it or set it to `null` to
  disable this check; `{}` enables strict empty declarations. See
  [secret inventory configuration](../configuration/workflow-topology-policy.md).
- `concurrencyPolicy`: owner id to the intended pending, cancellation, and
  scope behavior of that lock. See [concurrency configuration](../configuration/workflow-topology-policy-concurrency.md#concurrencypolicy).
- `forbidConcurrencyGroupCollisions`: when `true`, report workflows and jobs
  that share one concurrency group. `false` reports nothing.

Empty maps do not assert that every possible workflow is listed; each supplied
job or edge is checked and stale required targets are findings.
`unlockedWorkflowReasons` still documents workflows that have no lock.
`concurrencyPolicy` checks workflows and jobs that have one. The two options
are independent.

### `concurrencyPolicy`

For required fields, scope classification, and validation messages, see the
[concurrency configuration reference](../configuration/workflow-topology-policy-concurrency.md#concurrencypolicy).

### `forbidConcurrencyGroupCollisions`

`false` by default, which reports nothing. `true` reports every set of two or
more workflows or jobs whose `concurrency` group is the same text after
Unicode lowercasing, so `Deploy-Prod` and `deploy-prod` collide. Owners with
no `concurrency` block are ignored. Identical expression text collides,
including two copies of `${{ github.ref }}`.

A group that contains the exact placeholder `${{ github.workflow }}`, with
optional whitespace inside the braces, is compared only with owners in the
same workflow file. That context expands to the file's workflow name, so
`${{ github.workflow }}-${{ github.ref }}` in two files does not collide, while
the same text on a workflow lock and one of its jobs does. A longer expression
such as `${{ github.workflow || 'x' }}` is not partitioned.

`ci topology --format mermaid` still draws a separate lock for every group
that contains `${{ }}`, and joins only literal groups. This option treats
identical expression text as one lock. The diagram does not change.

```text
concurrency group collision: <lowered group>: <id1>, <id2>
```

```yaml
# .github/workflows/one.yml and .github/workflows/two.yml
concurrency:
  group: shared
# concurrency group collision: shared: .github/workflows/one.yml, .github/workflows/two.yml
```

Give each owner its own group. Include `${{ github.workflow }}` or another
value that differs per workflow, and use different text for a workflow lock
and a job lock in the same file.

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

For secret names, this workflow-level use is undeclared unless `DEPLOY_TOKEN`
is listed in the repository inventory. Whether top-level `env` is evaluated
with each job's `environment` secrets is not verified, so the undeclared
finding stays and that environment name is not reported unused.

```yaml
env:
  DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}"
```

## Fix

Add the missing topology edge or ordered step, or update the policy with the
intended graph and a reason for an intentionally unlocked workflow. For a
concurrency group collision, give each owner its own group. For an undeclared
secret, add its name at the repository or matching environment scope. List
organization secrets and Dependabot secrets under `repository`. Remove the
reference, or pass a composite action value through an input. Findings point to
the source file and line. Incomplete analysis is reported only when
`reportUnused` is on; it blocks the unused proof and does not hide a static
undeclared name. A `workflow_call` reference with no contract alias is still
undeclared when the name is absent from the repository inventory and every
environment inventory. Composite actions are scanned only when a workflow, or
an action already reached that way, references them through `uses: ./...`.
`uses: ./` is the repository-root action. Actions not reached that way are not
scanned, including actions used only by remote callers and actions invoked by
a path in a script. See the
[inventory guide](../configuration/workflow-topology-policy.md).

## Suppression

Prefer an `unlockedWorkflowReasons` entry for a deliberate exception. Suppress
one source finding with `no-mistakes-disable-line workflow-topology-policy` or
`no-mistakes-disable-next-line workflow-topology-policy`; use
`no-mistakes-disable-file workflow-topology-policy` only for a whole-file
exception such as a generated workflow.

## Related rules

[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) checks typecheck gates;
[`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks test path filters.
