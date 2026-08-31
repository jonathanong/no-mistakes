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
is reported as a configuration finding. `options.include` replaces the default
selection; common `include` and `exclude` add further rule-path scoping.

## Options and defaults

`options.include` defaults to `.github/workflows/**` and `.github/actions/**`.

## Valid example

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
```

## Counterexample

```yaml
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

## Suppression

Use `no-mistakes-disable-next-line no-sparse-checkout` for one documented
exception, or `no-mistakes-disable-file` for an intentionally exceptional file.

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) and
[`github-actions-job-timeouts`](github-actions-job-timeouts.md) enforce other
workflow reliability constraints.
