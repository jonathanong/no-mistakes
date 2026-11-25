# `workflow-topology-policy`

Declarative GitHub Actions topology assertions over the graph produced by
`ciTopology()` / `createWorkflowTopologyIndex()`. Configure inventory,
required and forbidden jobs and `needs` edges, artifact edges, exact
fan-in, reusable-workflow callers, step order, unlocked-workflow
reasons, secret-name declarations, the intended behavior of each
`concurrency:` block, shared concurrency groups, and forbidden trigger activity
types.

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
```

The following block is an extended reference of secret inventory and unlocked
workflows. The Options example above is the one Valid and Counterexample
execute.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      secretInventory:
        repository: [DEPLOY_TOKEN]
        environments:
          production: [RELEASE_KEY]
      unlockedWorkflowReasons:
        .github/workflows/ci.yml: "single-job lint has no overlapping work"
```

For secret names, this workflow-level use is undeclared unless `DEPLOY_TOKEN`
is listed in the repository inventory. Whether top-level `env` is evaluated
with each job's `environment` secrets is not verified, so the undeclared
finding stays and that environment name is not reported unused.

```yaml
env:
  DEPLOY_TOKEN: "${{ secrets.DEPLOY_TOKEN }}"
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
part of the delivery contract and should be checked as a graph. Ban explicit
trigger activity types when they would create unwanted workflow runs.

## What it catches/requires

Configured inventory and assertions must match workflow topology: required or
forbidden edges, exact fan-in, artifacts, callers, step order, declared secret
names, documented unlocked workflows, concurrency intent, shared groups, and
forbidden explicit trigger activity types.

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
  disable this check. `{}` enables the check with empty repository and
  environment lists and still exempts `GITHUB_TOKEN`. Set
  `builtInExemptions`, including to `[]`, to replace that default.
  `reportUnused` defaults to false: an unprovable secret reference is an
  ordinary finding only when it is true, and is omitted otherwise. See
  [secret inventory configuration](../configuration/workflow-topology-policy.md).
- `forbiddenActivityTypes`: entries with `types: string[]` and optional
  `events: string[]` banning exact explicit trigger activity types. Omitted
  `events` applies to all events; `events: []` applies to none. See
  [activity type configuration](../configuration/workflow-topology-policy-activity-types.md)
  for examples, ignored inputs, and suppression at the `on.<event>.types` key.
- `forbiddenEvents`: exact, case-sensitive event names banned on `on`. An
  omitted list or `[]` bans nothing. See
  [forbidden events](../configuration/workflow-topology-policy-forbidden-events.md).
- `forbidSecretsInherit`: when `true`, report reusable-workflow calls that set
  `secrets: inherit`. `false` reports nothing. See
  [secrets inherit](../configuration/workflow-topology-policy-secrets-inherit.md).
- `forbidReusableWorkflowConcurrency`: `false` by default. When enabled, ban
  workflow-level locks on workflows triggered by `workflow_call`, including
  inherited triggers and workflows with additional triggers. Job-level locks
  remain allowed. See the [concurrency reference](../configuration/workflow-topology-policy-concurrency.md#forbidreusableworkflowconcurrency)
  for counterexamples, rewrites, suppression, and analysis limits.
- `concurrencyPolicy`: owner id to the intended pending, cancellation, and
  scope behavior of that lock. See [concurrency configuration](../configuration/workflow-topology-policy-concurrency.md#concurrencypolicy).
- `forbidConcurrencyGroupCollisions`: when `true`, report workflows and jobs
  that share one concurrency group. `false` reports nothing.

Empty maps do not assert that every possible workflow is listed; each supplied
job or edge is checked and stale required targets are findings.
`unlockedWorkflowReasons` still documents workflows that have no lock.
`concurrencyPolicy` checks workflows and jobs that have one. The two options
are independent.

### `forbiddenActivityTypes`

For example, ban `edited` only on pull request triggers:

```yaml
forbiddenActivityTypes:
  - types: [edited]
    events: [pull_request, pull_request_target]
```

`on: {pull_request: {types: [opened, edited]}}` is a counterexample;
`on: {pull_request: {types: [opened]}}` is valid. Matching is exact and
case-sensitive, for string scalars and string sequence members. Scalar/list
`on`, missing `types`, and non-string values are ignored; defaults are not
expanded. Remove the forbidden type while keeping the allowed types. Findings
point to `on.<event>.types`; the standard rule-level `message` and suppression
directives apply at that key. See the
[configuration guide](../configuration/workflow-topology-policy-activity-types.md)
for YAML examples and event-filter defaults.

### `forbiddenEvents`

For example, ban `pull_request_target`:

```yaml
forbiddenEvents: [pull_request_target]
```

`on: pull_request_target`, `on: [push, pull_request_target]`, and
`on: {pull_request_target: {types: [opened]}}` are counterexamples.
`on: pull_request` in the same form is valid. Matching is exact and
case-sensitive. A mapping entry, including a null value, a quoted event
key, or a quoted `on` key, is reported on that event key. A comment
between `on:` and the event key does not move the finding. A sequence
item is reported on the item's line, including an item written on its own
line in a flow sequence. A one-line flow sequence shares the `on` line. A
scalar `on` value is reported on the `on` key. Duplicate names in one
workflow produce one finding. Remove the event. The standard rule-level
`message` and suppression directives apply at the reported key. A
`no-mistakes-disable-line` directive on the `on:` line of a null mapping
or a block sequence no longer applies, because the finding now lands on
the event key, and that directive is an `unused-disable-directive`
advisory. See the
[configuration guide](../configuration/workflow-topology-policy-forbidden-events.md).

### `forbidSecretsInherit`

`false` by default, which reports nothing. `true` reports a reusable-workflow
call whose `secrets` value is the exact string `inherit`, including a quoted
`"inherit"`, on local and remote `uses` calls.

```yaml
jobs:
  ship:
    uses: ./.github/workflows/orders.yml
    secrets: inherit
```

Map each secret by name. A missing `secrets` key and `secrets: {}` pass. A job
without a reusable-workflow `uses` is ignored. Findings point to the `secrets`
key. See the
[configuration guide](../configuration/workflow-topology-policy-secrets-inherit.md).

### `forbidReusableWorkflowConcurrency`

Move workflow-level concurrency in a reusable workflow to its caller or to a
job. Findings point to `concurrency`, irrespective of group text or cancellation.
See the [configuration guide](../configuration/workflow-topology-policy-concurrency.md#forbidreusableworkflowconcurrency).

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
# .github/workflows/ci.yml
name: ci
on: push
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
  test:
    needs: lint
    runs-on: ubuntu-latest
    steps:
      - run: pnpm test
```

## Counterexample

```yaml
# .github/workflows/ci.yml
name: ci
on: push
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
  test:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm test
```

## Fix

Job and relationship findings use the affected job key in its workflow; edge findings use the destination job. Workflow findings use the workflow document. If a configured job or workflow is absent, its workflow path remains the diagnostic location, with line 1 as the fallback. Line and next-line suppressions belong at that reported key.

For a forbidden activity type, remove that type from `on.<event>.types` and
retain the intended allowed types. Findings point to the `types` key; place
line and next-line suppressions there. See the
[configuration examples](../configuration/workflow-topology-policy-activity-types.md).

For a forbidden event, remove that event from `on`. Findings point to the
event key for a mapping entry, including a null value, and to the sequence
item for a list. A scalar `on: <event>` points at the `on` key. Place line
and next-line suppressions on that reported line. See the
[forbidden events guide](../configuration/workflow-topology-policy-forbidden-events.md).

For `secrets: inherit`, replace the value with an explicit mapping. Findings
point to the `secrets` key. A missing `secrets` key stays quiet, and
`forbidSecretsInherit: false` reports nothing. See the
[secrets inherit guide](../configuration/workflow-topology-policy-secrets-inherit.md).

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

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

- Parsed topology findings are ordinary findings. An unprovable secret reference is an ordinary finding only when `secretInventory.reportUnused` is true; otherwise it is omitted. Those diagnostics are not routed through `options.unanalyzable`.

## Suppression

Prefer an `unlockedWorkflowReasons` entry for a deliberate exception. Suppress
one source finding with `no-mistakes-disable-line workflow-topology-policy` or
`no-mistakes-disable-next-line workflow-topology-policy`; use
`no-mistakes-disable-file workflow-topology-policy` only for a whole-file
exception such as a generated workflow.

## Limitations

A workflow file that contains a YAML tag, or more than one YAML document,
has no source map. A forbidden-event finding in a tagged file is reported
on line 1. A multi-document file is not one workflow, so its events are
not reported.

The topology parser expands plain YAML 1.1 merge keys throughout the
document before reading triggers, concurrency, jobs, and steps. Quoted
`"<<"` keys remain literal. A merge that cannot be applied is reported as
a malformed workflow even when it appears only inside a job or step.

This expansion is an offline analysis convention. GitHub Actions supports
anchors and aliases under YAML 1.2. Use an alias for a whole mapping or
write explicit keys in workflows that GitHub runs.

When `forbidReusableWorkflowConcurrency` is `true`, every malformed-workflow
diagnostic is reported as a check failure, even for a file that is not a
`workflow_call` workflow. A file that cannot be parsed cannot be shown to be
non-reusable, so this over-inclusion is intended: it favors recall over a
missed reusable workflow. Repair the YAML, or exclude the file from
`ci.workflowDirs`, to clear it. A file with more than one YAML document is
malformed in this sense, so its concurrency is not read.

A directory symlink is not followed. `uses: ./actions/link`, where
`actions/link` is a symlink to `actions/real` and the undeclared secret exists
only in `actions/real/action.yml`, is not a finding.

Local `uses` paths collapse `.`, empty segments, and in-repo `..` before the
action file is resolved. `uses: ./.github/actions/../actions/deploy` is
`.github/actions/deploy`. A normalized path that is `..` or starts with `../`
leaves the repository and is not a local action, so its secrets are not
inventoried. A directory whose name only begins with `..`, such as
`..actions`, stays inside the repository.

## Related rules

[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) checks typecheck gates;
[`vitest-ci-path-coverage`](vitest-ci-path-coverage.md) checks test path filters.
