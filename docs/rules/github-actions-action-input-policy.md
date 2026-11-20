# `github-actions-action-input-policy`

## Why and when

Use this rule when one action family must receive exact `with:` values, or must
not receive selected inputs, on every ref. The checker does not hardcode action
names. A repository opts in by listing the families it cares about.

## What it requires

Each configured family matches `uses:` after the ref is removed, including a
commit SHA. Required inputs must be present as that scalar. Banned inputs must
be absent. A family that only bans inputs does not require those inputs.
Actions that are not listed stay silent. Other `with:` keys stay silent.

Remote `owner/repo` families match case-insensitively, so
`Example/Publish-Widget@V1` is `example/publish-widget`. A sub-path is a
different family: `example/publish-widget/restore` does not match
`example/publish-widget`, and the sub-path keeps the case written in `uses:`.
A path that starts with `.` or `/`, and any `uses:` value that contains `://`,
keeps its spelling.

The same policy applies to each step and to a job-level `uses:` call. A job
that sets both is checked for both. Composite actions are read from
`runs.steps` when `runs.using` is `composite`.

GitHub's workflow reader does not apply YAML merge keys. A `<<` key in `with:`
is an input named `<<`. Keys written under it are not inputs, so a required
input that appears only there is missing, and a banned input that appears only
there is not reported. A `<<` key on the step or job mapping itself also stays
as written and does not supply `uses` or `with`. An anchor that is the whole
`with:` value (`with: *base`) is expanded by the YAML parser and judged as that
mapping. A required value written as `${{ }}` is compared as that text, so it
is a mismatch unless the expression text is the required scalar. Suppress that
line when the expression is intentional. A `uses:` value that contains `${{ }}`
is not a concrete family and stays silent.

Numbers and booleans match their string forms, because GitHub Actions passes
`with:` values as strings. `1` and `"1"` are the same input. `true` and
`"true"` are the same input.

## Valid example

The widget publisher keeps artifacts for one day and replaces prior uploads.
The runtime setup reads a version file and turns the package-manager cache off.
Shipping items may set a name without setting `overwrite`.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: example/publish-widget@v1
        with:
          name: widgets
          path: widgets.log
          keep-days: 1
          replace: true
      - uses: example/setup-runtime@v2
        with:
          runtime-version-file: .runtime-version
          package-manager-cache: false
      - uses: example/ship-items@v1
        with:
          name: items
```

## Counterexample

Wrong required values, a banned input, and a missing required input all report.
A pinned SHA of the same family is judged the same way. `example/checkout-orders`
is not in the policy, so it stays silent.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: example/publish-widget@v1
        with:
          keep-days: 30
          replace: false
      - uses: example/publish-widget@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        with:
          keep-days: 30
          replace: true
      - uses: example/setup-runtime@v2
        with:
          cache: local
          runtime-version: "22"
          package-manager-cache: true
      - uses: example/checkout-orders@v1
        with:
          path: orders
```

## Fix

Set each required input to the configured scalar and delete banned inputs.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: example/publish-widget@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        with:
          keep-days: 1
          replace: true
      - uses: example/setup-runtime@v2
        with:
          runtime-version-file: .runtime-version
          package-manager-cache: false
```

## Suppression

Use a YAML comment on the input line that should stay as written.
`no-mistakes-disable-next-line github-actions-action-input-policy` applies to
the next line. `no-mistakes-disable-line github-actions-action-input-policy`
applies to the same line. `no-mistakes-disable-file github-actions-action-input-policy`
skips the file. An expression that supplies a required input is a mismatch;
put the line directive on that input when the expression is the value to keep.
Prefer narrowing `actions` when a family is outside the policy.

```yaml
steps:
  - uses: example/publish-widget@v1
    with:
      keep-days: 30 # no-mistakes-disable-line github-actions-action-input-policy
      replace: true
```

## Options and defaults

`actions` defaults to `[]`. An empty list reports nothing, so the rule is
opt-in. Each entry needs `uses`, the action family. A ref after `@` on that
name is ignored. `require` defaults to `{}` and maps an input name to a string,
number, or boolean. `ban` defaults to `[]` and lists input names that must be
absent. The same input cannot be both required and banned. Two entries for one
family merge; disagreeing required values are a configuration error.

`include` defaults to `.github/workflows/**/*.yml`,
`.github/workflows/**/*.yaml`, `.github/actions/**/action.yml`, and
`.github/actions/**/action.yaml`. An explicit list replaces that default.

```yaml
rules:
  - rule: github-actions-action-input-policy
    scope: repository
    options:
      include:
        - .github/workflows/**/*.yml
      actions:
        - uses: example/publish-widget
          require:
            keep-days: 1
            replace: true
        - uses: example/setup-runtime
          ban:
            - cache
            - runtime-version
          require:
            runtime-version-file: .runtime-version
            package-manager-cache: false
        - uses: example/ship-items
          ban:
            - overwrite
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Invalid YAML is skipped and produces no finding. A `uses:` value that contains
`${{ }}` is not a concrete action, so that step or job stays silent. A required
input set to a `${{ }}` expression is a definite mismatch when that text is not
the required scalar. The checker does not evaluate expressions. A `<<` key is
not expanded, so keys nested under it are not inputs. A file that cannot be
read is skipped and produces no finding.

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) requires a commit
SHA on `uses:`. [`github-actions-action-timeout-pair`](github-actions-action-timeout-pair.md)
requires paired timeouts for configured actions.
[`github-actions-composite-step-schema`](github-actions-composite-step-schema.md)
checks the keys of composite steps this rule also reads.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
