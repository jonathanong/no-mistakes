# `github-actions-checkout-credentials`

## Why and when

Use this rule when a workflow checks out a repository with `actions/checkout`.
The action persists the job `GITHUB_TOKEN` in the git credential store unless
`with.persist-credentials` is `false`. A later step can then push, publish, or
call the GitHub API with that token. The default is to persist credentials, so
an omitted input is not safe when the token can write.

The rule reports persisted credentials under declared write-capable bounds,
and inputs it cannot analyze.
Action files that cannot grant a token are skipped.

## What it catches

The rule scans `jobs.*.steps` in `.github/workflows/*.{yml,yaml}`. A file in
a subdirectory of `.github/workflows/` is not a workflow GitHub runs, so it
is ignored. Composite and other action files under
`.github/actions/**/action.{yml,yaml}` are read and then skipped: those files
have no job permissions of their own, and a pasted `permissions` or
`workflow_call` key is not applied by GitHub.

Only actual `actions/checkout` steps are judged: a valid `pull_request_target`
workflow without a checkout produces no finding. It flags a checkout when both are true:

- `with.persist-credentials` is omitted or a literal true (`true` or the
  string `true` in any letter case). A null value is unanalyzable. YAML treats
  `true` and `false` (any letter case) as booleans. The strings `yes`, `no`,
  `on`, and `off` are not booleans here, so they are not proof of false.
- The selected declared permission bound may allow `GITHUB_TOKEN` writes.

The bound is the job `permissions` block when that key is present. A job
block replaces the workflow block; it does not merge with it. Otherwise the
workflow `permissions` block is used. A missing workflow-level block is not
unknown when the checkout job declares its own permissions, including in a
reusable workflow. `write-all` counts as write. A scope
counts when its value is the string `write` (surrounding spaces ignored). The
write scopes are `actions`, `artifact-metadata`, `attestations`, `checks`,
`code-quality`, `contents`, `copilot-requests`, `deployments`, `discussions`,
`issues`, `packages`, `pages`, `pull-requests`, `security-events`, and
`statuses`.

`id-token: write` is the identity token, not a git credential. On its own it
does not count. `vulnerability-alerts: write` is not a valid grant.
Unrecognized scope names are not proof, and they do not hide a known write
next to them. The finding names the first proven write scope in source order.

For the default `GITHUB_TOKEN`, a reusable workflow's literal job or workflow
permissions are a known declared maximum. A caller can lower the token's actual access, but that does not make
the maximum unknown. A write-capable maximum with omitted or literal true
persistence is an ordinary unsafe finding under every policy. A read-only or
empty maximum is safe with literal or omitted persistence and the default
token. Findings describe the declared maximum, not proof of the caller's
actual runtime grant. A null or unsupported permissions value remains unknown.

If `permissions` is absent at both job and workflow levels, the rule does not
guess GitHub's default token,
including for `pull_request_target` and for `workflow_call`, unless
`assumeWriteWhenUnspecified` is true. Without that assumption the permissions
are unanalyzable. Expression-valued or otherwise unknown permissions and
persist inputs, non-null non-mapping `with`, and custom `with.token` use the
shared unanalyzable policy unless persistence is literal false. This includes
`${{ secrets.GITHUB_TOKEN }}`: a custom token cannot establish that a
credential is safe. A null `with` is treated like an omitted `with`.
Invalid YAML also uses that policy and includes the parser error and location.

`persist-credentials: false` is allowed even when the token can write. The
checkout name is `actions/checkout` with an optional ref and trailing slash.
`actions/checkout/extra`, a local path, and `docker://` are different actions.

## Options

`assumeWriteWhenUnspecified` defaults to `false`. Set it only for a repository
or organization whose default `GITHUB_TOKEN` can write. When it is true, a
missing `permissions` key at both levels, including on `workflow_call`, is
judged as write. Literal declared bounds are checked independently of this
option; a `workflow_call` write bound with persistence remains an ordinary
unsafe finding. A null `permissions:` value
stays unknown when the option is on. Unknown option keys are rejected.
Listing the rule is what turns it on. Product defaults do not enable it.

```yaml
rules:
  - rule: github-actions-checkout-credentials
    scope: repository
    options:
      assumeWriteWhenUnspecified: true
      unanalyzable: report
```

## Upgrading

This changes the default result of an already enabled rule. Unknown checkout
credentials used to be silently skipped; `options.unanalyzable` now defaults
to `report`, which emits failing findings. For checkouts that might persist
credentials, review these previously silent cases:

- No `permissions` key at either job or workflow level: repository or
  organization defaults are unknown, and
  `pull_request_target` alone does not prove the actual token grant.
- A `workflow_call` write declaration: its maximum is known even when the
  caller restricts runtime access. Persistence under that maximum is an ordinary
  unsafe finding; `advise` and `ignore` do not hide it.
- A custom `with.token`, such as a PAT or `${{ secrets.GITHUB_TOKEN }}`:
  job `GITHUB_TOKEN` permissions do not establish a custom token's permissions.
- Expression-valued `persist-credentials`: its runtime value cannot be
  established statically. Other unsupported input values are also unknown.
- Invalid workflow YAML: the parser cannot establish the checkout inputs
  and permissions.

The [reviewed unchanged old `abstain` snapshot][reviewed-abstain] went from
exit 0 with no findings on the base to exit 1 with 36 findings on the head.
That count describes that snapshot, not a fixed count for other repositories.
The current matcher fixture explicitly uses `ignore` to omit uncertainties.
Its declared reusable write bounds still produce ordinary findings; that
policy does not preserve the earlier default results.

Set `options.unanalyzable: ignore` to skip unknown inputs as before, or use
`advise` to list them as non-failing advisories in CLI and Node check reports.
The standalone Rust findings-only API omits advisories. Definite persisted-write
violations remain failures under every policy. In particular,
`assumeWriteWhenUnspecified: true` makes missing permissions a definite write
assumption, including on `workflow_call`; `advise` and `ignore` do not hide it.
When a job declares its own read-only bound, an absent workflow-level block
does not trigger this assumption. Valid workflows without checkout steps are not
permission findings, including on `pull_request_target`.
For a single accepted case, use a [line suppression](#suppression) on the
checkout `uses` line instead of changing the policy for the whole rule.

Prefer a static rewrite such as `with.persist-credentials: false`. An unknown
persistence input remains unanalyzable even with literal read-only permissions;
this conservative choice is intentional. See the
[migration note](../migrations/github-actions-checkout-credentials-unanalyzable.md)
for configuration and suppression examples.

[reviewed-abstain]: https://github.com/jonathanong/no-mistakes/pull/2697#issuecomment-6383196957

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
when the job does not need to write. For reusable workflows, use literal
read-only permissions or disable persistence; a declared write maximum remains
unsafe with persistence even when the caller can lower runtime access.
With `assumeWriteWhenUnspecified`, missing
permissions are a definite violation when credentials persist. A null
`persist-credentials` value remains unanalyzable.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

`options.unanalyzable` uses the shared policy: `report` (default) emits a
failing finding, `advise` emits a non-failing advisory, and `ignore` omits
uncertain results. Repository `unanalyzable` supplies the default.
Definite violations remain failing under every policy.

Expressions or unsupported values in `persist-credentials`, non-null
non-mapping `with`, custom `with.token`, unknown permission scopes/access values,
and permissions absent at both job and workflow levels are unanalyzable.
For the default `GITHUB_TOKEN`, literal reusable write bounds with omitted or
literal true persistence are ordinary findings. Custom tokens and unknown
persistence still follow their independent unanalyzable handling.
Known write grants still establish a violation even beside unknown scopes.
Literal `persist-credentials: false` is safe regardless of token or permissions.
Read-only or empty permissions without a custom token remain safe when
persistence is literal or omitted. An unknown persistence input still follows
the unanalyzable policy even with literal read-only permissions. A null `with`
is treated like omitted `with`.

Diagnostics name the file, job, step, and a static rewrite: set
`with.persist-credentials: false`, or use the default token with literal
read-only permissions. Invalid YAML reports its parser error and line;
job and step are `unknown` because the document could not be parsed.
Repair the YAML before relying on a clean result. Suppression comments apply
to both findings and advisories; malformed YAML supports file suppression
and line suppression at the reported parser-error location.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-checkout-credentials`
suppresses the next line. `no-mistakes-disable-line github-actions-checkout-credentials`
suppresses the same line. `no-mistakes-disable-file github-actions-checkout-credentials`
in the leading comment block suppresses the file. The standalone Rust `check()`
applies these directives too; its findings-only result omits advisories.

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
