# `package-json-registry-only`

Requires package registry settings to match configured policy.

```yaml
rules:
  - rule: package-json-registry-only
    scope: repository
    options:
      registry: "https://registry.npmjs.org/"
```

Counterexample: package metadata pointing at an unapproved registry.

Fix: update package metadata or the policy.
