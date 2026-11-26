# `no-sparse-checkout`

```yaml
rules:
  - rule: no-sparse-checkout
    scope: repository
```

## Why and when

Enable this rule when CI must analyze and validate the complete repository.
Sparse Git checkouts can silently omit inputs required by a workflow.

## What it catches

The rule parses Git-visible YAML files under `.github/workflows/**` and
`.github/actions/**` by default. It reports `sparse-checkout` and
`sparse-checkout-cone-mode` only in a `with:` mapping for an
`actions/checkout@…` workflow or composite-action step. Malformed selected YAML
produces a finding. A nonempty `options.include` replaces the default
selection. Common `include` and `exclude` add further rule-path scoping.

## Options and defaults

`options.include` uses `.github/workflows/**` and `.github/actions/**`
when omitted or empty. [Include selection](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/no_sparse_checkout.rs#L83-L96).

## Valid example

```yaml
name: ci
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
```

## Counterexample

```yaml
name: ci
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          sparse-checkout: scripts
```

## Fix

Remove `sparse-checkout` and `sparse-checkout-cone-mode` from the checkout
step. To scan an additional Git-visible YAML location, configure it explicitly:

```yaml
options:
  include: ["ci/workflows/**"]
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Unreadable files and steps without a statically recognized checkout and
a `with:` mapping are skipped. The rule does not use `options.unanalyzable`. [File reading](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/no_sparse_checkout/scan.rs#L14-L32) and [step scanning](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/no_sparse_checkout/scan.rs#L77-L91).

## Suppression

Use `no-mistakes-disable-next-line no-sparse-checkout` for one documented
exception, or `no-mistakes-disable-file` for an intentionally exceptional file.

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) and
[`github-actions-job-timeouts`](github-actions-job-timeouts.md) enforce other
workflow reliability constraints.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
