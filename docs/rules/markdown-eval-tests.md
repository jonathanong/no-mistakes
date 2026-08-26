# `markdown-eval-tests`

## Why and when

Use this rule when tests must not execute shell snippets copied from Markdown,
where prose changes could unexpectedly become executable behavior.

## What it catches

It catches evaluation of Markdown shell blocks outside the exact-path allowlist.

## Options

`include` selects test files and defaults to empty, which disables the rule
until files are chosen. `allow` defaults to empty and lists exact test-file
exceptions, not Markdown paths or globs.

## Valid example

A test that asserts rendered Markdown text without evaluating a shell fence
passes.

## Related rules

[`markdown-mermaid-validation`](markdown-mermaid-validation.md) validates a
safe, declarative fence language; [`shellcheck-runner`](shellcheck-runner.md)
checks real shell files.

## Suppression

Use file suppression only for the guard's own controlled Markdown-evaluation
fixture. Prefer an exact `allow` entry for a reviewed exception.

Flags test files that look like they read a `.md` file and `eval` an extracted
shell block inside a spawned `bash`/`sh`/`zsh` process. That pattern is a
per-assertion cost outlier; prefer spawn-free `readFileSync` / `.toContain`
checks unless the file is an explicit, reviewable exception.

```yaml
rules:
  - rule: markdown-eval-tests
    scope: repository
    options:
      include: ["**/*.test.mts"]
      allow:
        - opentofu/reference-live-runbook-execution-safety.test.mts
```

A file matches only when all three heuristics are present in the whole file: a
`.md` string literal, a `spawn`/`exec` of `bash`/`sh`/`zsh`, and `\beval\b`.
`allow` is exact relative paths, not directory globs. Unused allow entries are
findings.

Counterexample: a test interpolates a markdown path, spawns bash, and calls
`eval`.

```ts
import { execFileSync } from "node:child_process";
execFileSync("bash", ["-c", "eval \"$block\""]);
const doc = "runbook.md";
```

Fix: assert on file contents without spawning a shell, or add this exact
relative path to `options.allow` with a rationale in the PR.

```ts
import { readFileSync } from "node:fs";
const body = readFileSync("runbook.md", "utf8");
expect(body).toContain("tofu apply");
```

Use `no-mistakes-disable-file markdown-eval-tests` only when the file is the
guard's own test fixture, not as a substitute for `allow`.
