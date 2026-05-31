# `no-git-identity-mutation`

Bans scripts that mutate git user identity.

```yaml
rules:
  - rule: no-git-identity-mutation
    scope: repository
```

Counterexample: `git config user.email bot@example.com` in setup scripts.

Fix: read git identity when needed, but configure identity outside repository
scripts.
