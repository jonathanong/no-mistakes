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
