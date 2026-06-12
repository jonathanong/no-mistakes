# `structured-config-policy`

Requires or bans dotted keys in structured YAML or JSON config files.

```yaml
rules:
  - rule: structured-config-policy
    scope: repository
    options:
      policies:
        - files: [app.yml]
          requiredKeys: [runtime.version]
          bannedKeys: [legacy.enabled]
```

Counterexample: a config file omits a required key or still contains a banned
legacy key.

Fix: add the required key, remove the banned key, or narrow the file glob to the
configs where the policy applies.
