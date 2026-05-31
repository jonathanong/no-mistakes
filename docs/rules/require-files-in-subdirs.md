# `require-files-in-subdirs`

Requires each matching subdirectory to contain configured files.

```yaml
rules:
  - rule: require-files-in-subdirs
    scope: repository
    options:
      roots: [packages]
      files: [README.md, package.json]
```

Counterexample: `packages/api/` without a `README.md`.

Fix: add the required file or exclude the directory.
