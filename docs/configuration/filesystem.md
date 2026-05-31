# Filesystem Configuration

`filesystem.skipDirectories` removes directories from discovery unless a rule
target root intentionally preserves them.

```yaml
filesystem:
  skipDirectories:
    - node_modules
    - dist
```

Filesystem rules discover tracked files once and reuse that file list across
enabled rules.
