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

`bannedDomains` must be supplied even when options are otherwise omitted; an
explicit empty list disables domain bans. HTML named and decimal/hex numeric
entities are decoded in addresses, including domain labels and dots. Findings
retain the original source spelling and line. `allowedEmailPatterns` matches
that original spelling, including HTML or URL encoding.

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

- A selected file with invalid UTF-8 is decoded lossily and scanned, so a disallowed email domain next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding. `options.unanalyzable` does not change either result. Literal email domains in readable test files are ordinary findings.

## Suppression

Use `no-mistakes-disable-next-line test-email-domain-policy` on the one line;
avoid file-wide suppression so the domain remains banned elsewhere.

## Related rules

[`test-no-dependency-pins`](test-no-dependency-pins.md) prevents brittle test
assertions; [`test-no-unmocked-dynamic-imports`](test-no-unmocked-dynamic-imports.md)
protects test isolation.

Unused `allowedEmailPatterns` entries produce `unused-allow-entry` advisories by default
when the complete relevant file scope was successfully checked. Remove stale
entries. Every overlapping exception that suppresses a raw finding is used;
matching text that would not violate the rule does not use an exception.
Partial scopes and failed reads or parses do not prove an entry stale.

Repository `reportUnusedAllowEntries: report` makes these diagnostics fail;
`ignore` omits them. See [allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).
