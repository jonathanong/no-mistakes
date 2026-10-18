# `github-actions-checkout-credentials`

## Why and when

Use this rule when a workflow checks out a repository with `actions/checkout`.
The action persists the job `GITHUB_TOKEN` in the git credential store unless
`with.persist-credentials` is `false`. A later step can then push, publish, or
call the GitHub API with that token. The default is to persist credentials, so
an omitted input is not safe when the token can write.

The rule reports only what it can prove. Unknown permissions, unknown inputs,
and action files that cannot grant a token are not findings.

## What it catches

The rule scans `jobs.*.steps` in `.github/workflows/*.{yml,yaml}`. Composite
and other action files under `.github/actions/**/action.{yml,yaml}` are read
and then skipped: those files have no job permissions of their own, and a
pasted `permissions` or `workflow_call` key is not applied by GitHub.

It flags an `actions/checkout` step when both of these are true:

- `with.persist-credentials` is omitted, null, or a literal true (`true` or
  the string `true` in any letter case). YAML treats `true` and `false` (any
  letter case) as booleans. The strings `yes`, `no`, `on`, and `off` are not
  booleans here, so they are not proof of false.
- The job `GITHUB_TOKEN` is proven to be able to write.

Write access is the job `permissions` block when that key is present. A job
block replaces the workflow block; it does not merge with it. Otherwise the
workflow `permissions` block is used. `write-all` counts as write. A scope
counts when its value is the string `write` (surrounding spaces ignored). The
write scopes are `actions`, `artifact-metadata`, `attestations`, `checks`,
`code-quality`, `contents`, `copilot-requests`, `deployments`, `discussions`,
`issues`, `packages`, `pages`, `pull-requests`, `security-events`, and
`statuses`.

`id-token: write` is the identity token, not a git credential. On its own it
does not count. `vulnerability-alerts: write` is not a valid grant.
Unrecognized scope names are not proof, and they do not hide a known write
next to them. The finding names the first proven write scope in source order.

A reusable workflow (`on` includes `workflow_call`, alone or with other
events) that has no `permissions` key inherits the caller token and counts as
write-capable. A present `permissions` key, including a null or empty value,
is not that exception. Empty permissions are not write.

If `permissions` is absent and the workflow is not `workflow_call`, the rule
does not guess GitHub's default token, including for `pull_request_target`.
That is not a finding. Expressions and other values that are not literal true
or false for `persist-credentials`, and permission values that are not literal
`read`, `none`, or `write`, are not findings. An explicit `with.token` is not
judged, because it is not proven to be the job `GITHUB_TOKEN`. Invalid YAML is
not a finding.

`persist-credentials: false` is allowed even when the token can write. The
checkout name is `actions/checkout` with an optional ref and trailing slash.
`actions/checkout/extra`, a local path, and `docker://` are different actions.

## Options

This rule has no options. There is no configurable default. Listing the rule
is what turns it on. Product defaults do not enable it.

```yaml
rules:
  - rule: github-actions-checkout-credentials
    scope: repository
```

## Valid example

Drop the credential when the token can write, or keep the token read-only.

```yaml
permissions:
  contents: read
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
```

A reusable workflow that sets read permissions does not inherit a write token:

```yaml
on: workflow_call
permissions:
  contents: read
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
```

## Counterexample and fix

This checkout persists a contents write token because the input is omitted:

```yaml
on: push
permissions:
  contents: write
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
```

Set the input to false:

```yaml
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
```

The same fix applies to `permissions: write-all`, to a job block that grants
any write scope above, and to a `workflow_call` workflow that sets no
permissions. Narrowing the token to `contents: read` is the other fix when
the job does not need to write.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-checkout-credentials`
suppresses the next line. `no-mistakes-disable-line github-actions-checkout-credentials`
suppresses the same line. `no-mistakes-disable-file github-actions-checkout-credentials`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-checkout-credentials
- uses: actions/checkout@v4
- uses: actions/checkout@v4 # no-mistakes-disable-line github-actions-checkout-credentials
```

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) pins action code.
[`github-actions-script-expressions`](github-actions-script-expressions.md)
rejects untrusted expressions inside scripts. This rule only judges persisted
checkout credentials.
