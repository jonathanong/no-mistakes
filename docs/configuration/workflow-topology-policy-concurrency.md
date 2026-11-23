# Workflow topology concurrency options

This page documents the `concurrencyPolicy`, `forbidConcurrencyGroupCollisions`,
and `forbidReusableWorkflowConcurrency` options of the
[`workflow-topology-policy` rule](../rules/workflow-topology-policy.md).

## `concurrencyPolicy`

Checked only when the map is non-empty. Every workflow or job whose
`concurrency` is present needs a row, and every row needs an owner that still
has a lock.
An empty map checks nothing. An owner id is a workflow path or a job id
`<path>#<key>`; quote job ids in YAML because `#` starts a comment.

| Field | Values | Meaning |
| --- | --- | --- |
| `pending` | `coalesce-latest`, `fifo` | `queue: max` is `fifo`. Any other queue, including the default, is `coalesce-latest`. |
| `cancellation` | `cancel-running`, `retain-running`, `conditional` | `cancel-in-progress: true` is `cancel-running`, `false` is `retain-running`, and an expression string is `conditional`. |
| `scope` | non-empty list | `pull-request`, `ref`, `sha`, `run`, `event`, `input-resource`, `fixed-resource`. |

Unknown names, a missing field, an empty `scope`, a duplicate scope entry, or
`fixed-resource` combined with another scope is a config error:
`workflow-topology-policy option concurrencyPolicy: ...`.

`cancel-in-progress` expressions must be one complete `${{ ... }}` value after
trimming. Anything else, such as `github.ref == main`, reports
`conditional cancel-in-progress expression invalid: <id>: <value>`.

Scope is read from `concurrency.effective.group`. Each `github.*` or `inputs.*`
reference is classified below. References are de-duplicated and ordered
`pull-request`, `ref`, `sha`, `run`, `event`, `input-resource`.
`github.workflow` and `github.run_attempt` are ignored. `fixed-resource` means
the group has no `github.*` or `inputs.*` reference. Any other reference is
`unsupported:<reference>` and cannot match a declared scope.

| Scope | References |
| --- | --- |
| `pull-request` | `github.event.pull_request.number`, `github.event.workflow_run.pull_requests` |
| `sha` | `github.sha`, `github.event.pull_request.head.sha`, `github.event.workflow_run.head_sha` |
| `ref` | `github.ref`, `github.ref_name`, `github.event.workflow_run.head_branch` |
| `run` | `github.run_id` |
| `input-resource` | `inputs.*`, `github.event.inputs`, `github.event.inputs.*` |
| `event` | `github.event_name`, `github.event.action`, `github.event.label.name`, `github.event.issue.number`, `github.event.schedule`, `github.event.workflow_run.event`, `github.event.workflow_run.id`, `github.event.workflow_run.workflow_id` |

The most specific table entry wins, so
`github.event.workflow_run.head_branch` is `ref` and
`github.event.workflow_run.head_sha` is `sha`. A reference the table does not
list, such as `github.actor`, is reported rather than guessed.

```text
concurrency intent missing: <id>
concurrency intent stale: <id>
concurrency pending mismatch: <id>: expected <declared>, got <actual>
concurrency cancellation mismatch: <id>: expected <declared>, got <actual>
concurrency scope mismatch: <id>: expected <a, b>, got <c, d>
conditional cancel-in-progress expression invalid: <id>: <value>
```

Quote `group` and `cancel-in-progress` values that contain `{`, and quote job-id
keys. With `cancellation: conditional` and `scope: [pull-request, sha]`
declared for `.github/workflows/ci.yml`:

```yaml
concurrency:
  group: "ci-${{ github.event.pull_request.number }}-${{ github.event.pull_request.head.sha }}"
  cancel-in-progress: true
# concurrency cancellation mismatch: .github/workflows/ci.yml: expected conditional, got cancel-running
```

## `forbidConcurrencyGroupCollisions`

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

## `forbidReusableWorkflowConcurrency`

`false` by default. Set it to `true` to report workflow-level `concurrency`
in any workflow whose triggers include `workflow_call`. This applies to
scalar and mapping locks, independent of the group expression or
`cancel-in-progress` value. Other triggers on the same workflow do not exempt it.
Job-level concurrency is allowed, as are workflow locks on ordinary callers.

```yaml
rules:
  - rule: workflow-topology-policy
    scope: repository
    options:
      forbidReusableWorkflowConcurrency: true
```

A reusable workflow holding its own workflow lock can wait on or cancel its
caller when their group values coincide. This counterexample reports on the
`concurrency` key:

```yaml
on: workflow_call
concurrency:
  group: "${{ github.workflow }}-${{ github.ref }}"
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo example
```

Move the lock to the caller, or use job-level concurrency instead. For example,
this reusable workflow is accepted:

```yaml
on: workflow_call
jobs:
  build:
    runs-on: ubuntu-latest
    concurrency:
      group: "example-build-${{ github.ref }}"
      cancel-in-progress: false
    steps:
      - run: echo example
```

The lock intent map cannot require absence of a workflow lock, and collision
detection only compares group text. Neither replaces this trigger-based ban.
The exported `WorkflowTopologyPolicyOptions` type describes the YAML options
used by CLI `check`, async `check({ root, config })`, and `analyzeProject()`
check reports; no extra invocation argument is needed.

Use the standard custom `message` or suppress an intentional exception at the
reported workflow-level key:

```yaml
on: workflow_call
# no-mistakes-disable-next-line workflow-topology-policy -- isolated lock for this example
concurrency: example-isolated
```

`no-mistakes-disable-line` on that key and `no-mistakes-disable-file` before the
first code token also apply. Suppression does not require disabling job locks.

### Limits

The ban uses the declared `workflow_call` trigger, including a mapping entry
with a null value or a trigger inherited through a YAML merge key. The ban
applies even when the workflow has no callers. Workflows whose YAML cannot be
parsed are skipped by the ban. Workflow topology still reports its
`malformed-workflow` diagnostic for those files.
