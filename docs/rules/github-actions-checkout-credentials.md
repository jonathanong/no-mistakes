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

The rule scans `jobs.*.steps` in `.github/workflows/*.{yml,yaml}`. A file in
a subdirectory of `.github/workflows/` is not a workflow GitHub runs, so it
is ignored. Composite and other action files under
`.github/actions/**/action.{yml,yaml}` are read and then skipped: those files
have no job permissions of their own, and a pasted `permissions` or
`workflow_call` key is not applied by GitHub.

It flags an `actions/checkout` step when both of these are true:

- `with.persist-credentials` is omitted or a literal true (`true` or the
  string `true` in any letter case). A null value is not judged, so it is
  not a finding. YAML treats `true` and `false` (any letter case) as
  booleans. The strings `yes`, `no`, `on`, and `off` are not booleans here,
  so they are not proof of false.
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

A file whose `on` includes `workflow_call`, alone or with other events,
cannot prove a write token. The caller can only pass the same or a more
restrictive `GITHUB_TOKEN`, and this rule does not read the caller. A write
grant in that file is not a finding. Empty permissions are not write. A null
`permissions` value stays unknown. `assumeWriteWhenUnspecified` does not
change any of those cases.

If `permissions` is absent, the rule does not guess GitHub's default token,
including for `pull_request_target` and for `workflow_call`, unless
`assumeWriteWhenUnspecified` is true. The default is false, so a missing key
is not a finding. Expressions and other values that are not literal true
or false for `persist-credentials`, and permission values that are not literal
`read`, `none`, or `write`, are not findings. An explicit `with.token` is not
judged, because it is not proven to be the job `GITHUB_TOKEN`. Invalid YAML is
not a finding.

`persist-credentials: false` is allowed even when the token can write. The
checkout name is `actions/checkout` with an optional ref and trailing slash.
`actions/checkout/extra`, a local path, and `docker://` are different actions.

## Options

`assumeWriteWhenUnspecified` defaults to `false`. Set it only for a repository
or organization whose default `GITHUB_TOKEN` can write. When it is true, a
missing `permissions` key, including on `workflow_call`, is judged as write.
An explicit permissions value is unchanged, and a `workflow_call` file that
sets `contents: write` is still not a finding. A null `permissions:` value
stays unknown when the option is on. Unknown option keys are rejected.
Listing the rule is what turns it on. Product defaults do not enable it.

```yaml
rules:
  - rule: github-actions-checkout-credentials
    scope: repository
    options:
      assumeWriteWhenUnspecified: true
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

## Counterexample

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

## Fix

Set the input to false:

```yaml
      - uses: actions/checkout@v4
        with:
          persist-credentials: false
```

The same fix applies to `permissions: write-all` and to a job block that grants
any write scope above. Narrowing the token to `contents: read` is the other fix
when the job does not need to write. A `workflow_call` workflow is not a
finding when it sets `contents: write`: the caller can only pass the same or
a more restrictive token, and this rule does not read the caller. With
`assumeWriteWhenUnspecified`, a `workflow_call` file that omits `permissions`
is a finding, because the option treats that missing key as write. A null
`persist-credentials` value is not a finding either.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges the literal `persist-credentials` setting of parsed `actions/checkout` steps. Nothing is routed through `options.unanalyzable`.

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
rejects untrusted expressions inside scripts.
[`github-actions-download-integrity`](github-actions-download-integrity.md)
flags unverified `curl` and `wget` downloads. This rule only judges persisted
checkout credentials.
