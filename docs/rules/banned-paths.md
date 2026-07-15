# `banned-paths`

Bans tracked files whose repository-relative paths match configured globs. In a
Git worktree, the rule examines files present in the index and working tree. It
does not report untracked files, whether or not Git ignores them. A tracked file
remains eligible when a later ignore pattern matches it.

Outside a Git worktree, there is no index to define tracked files. The rule
falls back to the ignore-aware visible file set, applying `.gitignore` and
`.ignore` files like other automatic discovery.

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

Paths supplied directly to the programmatic matcher are authoritative. This
lets callers check a known path set without creating a Git repository.

Suppression caveat: findings report line 1 of the offending file, so prefer a
top-of-file `no-mistakes-disable-file banned-paths` directive for intentional
exceptions.
