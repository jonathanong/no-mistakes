# Filesystem Configuration

`filesystem.skipDirectories` removes directories from discovery unless a rule
target root intentionally preserves them.

```yaml
filesystem:
  skipDirectories:
    - node_modules
    - dist
```

Filesystem rules build one discovery snapshot and reuse its file inventories
across enabled rules. Most rules use the general Git-visible inventory, which
contains tracked files and visible untracked files. Repository-state rules such
as `banned-paths` use the snapshot's tracked-only inventory inside a Git
worktree. Outside Git, they use the ignore-aware visible inventory as a
fallback.
