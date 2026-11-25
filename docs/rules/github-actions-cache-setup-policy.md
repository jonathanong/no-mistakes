# `github-actions-cache-setup-policy`

Use this opt-in rule when a package-cache key follows an ordered template and
selected setup actions need tighter step deadlines than other workflow steps.
For example, a pnpm cache namespace can put the runner OS before the
architecture and append the lockfile digest expression.

## Options

- `caches` lists policies with `uses`, `path`, and `keyPattern`. The
  action family omits its ref. A path must match one entry in `with.path`
  after trimming whitespace, one leading `./`, and trailing slashes.
  Multiline strings and YAML sequences are supported. `keyPattern` is a
  Rust regular expression matched against the entire trimmed authored
  `with.key` string, including its GitHub Actions expression text.
- `setup` lists action families as `uses` and their `maxMinutes` from
  1 through 360. Matching steps need a positive YAML integer at or below
  that cap in `timeout-minutes`. Shorter deadlines pass.
- `include` replaces the default `.github/workflows/*.yml` and
  `.github/workflows/*.yaml` globs and `.github/actions/**/action.yml` or
  `action.yaml` under `.github/actions/`. The rule checks workflow steps,
  including children of `parallel`. Cache templates also apply to composite
  actions. Setup deadlines apply only to workflows, where GitHub accepts
  step-level timeouts. Reusable job calls are outside its scope.

Configure at least one cache or setup policy. Options reject blank selectors,
duplicate normalized selectors, invalid globs or regular expressions, and
out-of-range caps. Invalid options fail at config load with exit 2. Remote
action families use the same case matching as the action-input rule.

```yaml
rules:
  - rule: github-actions-cache-setup-policy
    scope: repository
    options:
      caches:
        - uses: actions/cache
          path: .cache/widgets
          keyPattern: 'pnpm-\$\{\{\s*runner\.os\s*\}\}-\$\{\{\s*runner\.arch\s*\}\}-\$\{\{\s*hashFiles\(''widgets\.lock''\)\s*\}\}'
      setup:
        - uses: pnpm/action-setup
          maxMinutes: 3
```

## Example

The cache key follows the configured order and the setup action has a two-minute deadline.

```yaml
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/cache@v4
        with:
          path: .cache/widgets
          key: pnpm-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('widgets.lock') }}
      - uses: pnpm/action-setup@v4
        timeout-minutes: 2
```

## Counterexample and fix

This key reverses the platform fields, and the setup action can take up to
five minutes. Put the OS before the architecture and set the deadline to
three minutes or less.

```yaml
steps:
  - uses: actions/cache@v4
    with:
      path: .cache/widgets
      key: pnpm-${{ runner.arch }}-${{ runner.os }}-${{ hashFiles('widgets.lock') }}
  - uses: pnpm/action-setup@v4
    timeout-minutes: 5
```

## Unanalyzable inputs

Unreadable or malformed YAML, indeterminate action or path selectors,
expression-valued deadlines, and cache keys hidden behind a whole-value
expression follow `options.unanalyzable` when the rule cannot establish
conformance. The default `report` fails, `advise` emits a non-failing
advisory, and `ignore` omits uncertainty. A definite key mismatch or
deadline violation still fails under all modes. The rule checks authored
templates and does not evaluate workflow expressions.

## Suppression

Use `no-mistakes-disable-line github-actions-cache-setup-policy` on the
reported field, or `no-mistakes-disable-next-line github-actions-cache-setup-policy`
above it. Use `no-mistakes-disable-file github-actions-cache-setup-policy`
at the top of an intentionally exempt workflow.

## Related rules

- [Action input policy](github-actions-action-input-policy.md) owns exact
  and allowed `with` values.
- [Cache key policy](github-actions-cache-key-policy.md) owns digest
  provenance, expression fragments, forbidden paths, and restore-prefix policy.
- [Action timeout pairs](github-actions-action-timeout-pair.md) own exact
  action timeouts and nested timeout inputs.
- [Step timeouts](github-actions-step-timeouts.md) owns workflow-wide caps.
