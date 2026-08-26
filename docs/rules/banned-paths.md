# `banned-paths`

## Why and when

Use this rule for migration or architecture boundaries expressed as path globs,
especially when a file must disappear rather than merely stop being imported.

## What it catches

It reports tracked or visible files matching a configured `bannedPaths` glob,
including repository-scoped matches below normal source-discovery skips.

## Options

Each `bannedPaths` entry requires `glob` and may set a custom `message`.
There are no defaults or other rule-specific options.

## Valid example

A repository without `web/pages/**` files passes the configuration below.

## Related rules

[`banned-renamed-files`](banned-renamed-files.md) is for legacy basenames;
[`file-extension-policy`](file-extension-policy.md) is for extension policy.

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

Repository-scoped bans inspect the repository inventory before source-analysis
directory skips are applied. Tracked matches under built-in skip directories
such as `fixtures`, `build`, `dist`, and `target` are therefore still reported.
Use the rule's `include` and `exclude` filters when the repository policy should
intentionally cover a narrower path set.

Suppression caveat: findings report line 1 of the offending file, so prefer a
top-of-file `no-mistakes-disable-file banned-paths` directive for intentional
exceptions.
