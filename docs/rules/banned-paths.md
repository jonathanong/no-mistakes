# `banned-paths`

Bans tracked files whose repository-relative paths match configured globs.

```yaml
rules:
  - rule: banned-paths
    scope: repository
    options:
      bannedPaths:
        - glob: web/pages/**
          message: Next.js pages router files are not allowed
        - glob: web/app/**/[topicType]/**
          message: use explicit routes per topic type
```

Counterexample: a repository keeps legacy route files such as `web/pages/index.tsx`
or dynamic route segments that the project has banned.

Fix: remove or rename the file so it no longer matches the configured path ban.

Suppression caveat: findings report line 1 of the offending file, so prefer a
top-of-file `no-mistakes-disable-file banned-paths` directive for intentional
exceptions.
