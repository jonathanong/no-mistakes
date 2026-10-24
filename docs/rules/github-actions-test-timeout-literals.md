# `github-actions-test-timeout-literals`

## Why and when

Use this rule when `github-actions-job-timeouts` owns timeout policy and tests
should verify relationships rather than duplicate policy literals.

## What it catches

It catches embedded timeout YAML literals and direct timeout equality assertions
in matching workflow tests.

## Options

`include` defaults to workflow `.test.mts` and `.test.ts` files. Each `allow`
entry requires `file`, exact trimmed `text`, and a non-empty `reason`; no other
rule-specific options exist.

## Valid example

A relational assertion compares values without restating the owned timeout
literal.

```ts
// .github/workflows/ci.test.mts
expect(step?.['timeout-minutes']).toBeLessThanOrEqual(job?.['timeout-minutes'])
```


## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) owns the
workflow timeout policy this rule protects from duplication.

Rejects workflow tests that restate `timeout-minutes` literals the job-timeouts
rule already owns. YAML fragments such as `timeout-minutes: 15` and equality
assertions against `['timeout-minutes']` or `.timeoutMinutes` are findings.

```yaml
rules:
  - rule: github-actions-test-timeout-literals
    scope: repository
    options:
      include: [".github/workflows/**/*.test.mts"]
```

An `allow` entry pins one exact test line. It is a finding when that line is
absent from the scanned file:

```yaml
rules:
  - rule: github-actions-test-timeout-literals
    scope: repository
    options:
      allow:
        - file: .github/workflows/ci.test.mts
          text: "timeout-minutes: 20"
          reason: pins fromJSON branch values the timeout rule cannot resolve
```

`include` defaults to `.github/workflows/**/*.test.mts` and `*.test.ts`.
`allow` keys are `${file}#${trimmed line}`. Empty `reason` is a finding.
Unused `allow` entries for scanned files are findings.

Counterexample: a workflow test embeds `timeout-minutes: 15` in a string, or
asserts `expect(step?.['timeout-minutes']).toBe(10)`.

```ts
expect(workflowSource).toContain('timeout-minutes: 15')
expect(step?.['timeout-minutes']).toBe(10)
```

Fix: delete the assertion, or add a reasoned `allow` entry whose `text` is the
trimmed source line.

```ts
expect(step?.['timeout-minutes']).toBeLessThanOrEqual(job?.['timeout-minutes'])
```

## Counterexample

A workflow test embeds `timeout-minutes: 15` in a string, or asserts equality
against the owned literal.

```ts
// .github/workflows/ci.test.mts
expect(workflowSource).toContain('timeout-minutes: 15')
expect(step?.['timeout-minutes']).toBe(10)
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal test-timeout values in parsed workflow steps. Nothing is routed through `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-next-line github-actions-test-timeout-literals` or
`no-mistakes-disable-file` for a one-off test exception.
Line and next-line suppressions require an exact finding line.
Use file suppression when the exception applies to the full file.
