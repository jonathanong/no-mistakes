# `github-actions-test-timeout-literals`

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

Use `no-mistakes-disable-next-line github-actions-test-timeout-literals` or
`no-mistakes-disable-file` for a one-off test exception.
