# `require-files-in-subdirs`

Requires each matching subdirectory to contain configured files.

```yaml
rules:
  - rule: require-files-in-subdirs
    scope: repository
    options:
      packages:
        - root: packages
          requiredFiles: [README.md, package.json]
          requireAnyOf:
            - [src/index.ts, src/index.mts]
```

Counterexample: `packages/api/` without a `README.md`.

Fix: add the required file or exclude the directory.
