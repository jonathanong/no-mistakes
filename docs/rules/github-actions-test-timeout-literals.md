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

The relational assertion in the fix below is valid because it compares values
without restating the owned timeout literal.

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

## Suppression

Use `no-mistakes-disable-next-line github-actions-test-timeout-literals` or
`no-mistakes-disable-file` for a one-off test exception.
Line and next-line suppressions require an exact finding line.
Use file suppression when the exception applies to the full file.
