# `test-email-domain-policy`

Bans configured email domains in tracked test fixtures and documentation files.

```yaml
rules:
  - rule: test-email-domain-policy
    projects: [web]
    options:
      bannedDomains: [example.com]
      allowedEmailPatterns:
        - '(?i)^tests(?:\+[a-z0-9._%${}-]+|%2b[a-z0-9._%${}-]+)(?:@|%40)example\.test$'
      replacement: tests+<hash>@example.test
      extensions: [.md, .mts, .ts, .txt]
```

Counterexample: a test fixture contains `person@example.com`, even if the
address is URL-encoded as `%40example.com`.

Fix: replace the address with the configured `replacement`, add a narrowly
scoped `allowedEmailPatterns` entry for the intentional case, or remove the
fixture content entirely.

Suppression caveat: suppress only a single line for a one-off fixture exception.
Keep the suppression local so the banned domain still applies everywhere else in
the file set.

## Why and when

Use this rule to keep tracked fixtures and docs from sending mail to real
domains while preserving deterministic, obviously synthetic test addresses.

## What it catches/requires

Configured banned domains must not occur in selected files, including URL- or
HTML-encoded forms, unless an allowed pattern matches.

## Options and defaults

`bannedDomains` is required. `allowedEmailPatterns` and `replacement` are
optional; `extensions` defaults to the supported text/test extensions when not
specified. Project and common include/exclude filters also apply.

## Valid example

```text
tests+profile@example.test
```

## Counterexample

```text
owner@example.com
```

## Fix

Replace the address with the configured synthetic replacement or add a narrowly
scoped allow pattern for a documented, intentional fixture.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal email domains in test files. Nothing is routed through `options.unanalyzable`.

## Suppression

Use `no-mistakes-disable-next-line test-email-domain-policy` on the one line;
avoid file-wide suppression so the domain remains banned elsewhere.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) prevents brittle test
assertions; [`test-no-unmocked-dynamic-imports`](test-no-unmocked-dynamic-imports.md)
protects test isolation.
