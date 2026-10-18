# Workflow topology concurrency options

This page documents the `concurrencyPolicy` and
`forbidConcurrencyGroupCollisions` options of the
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
