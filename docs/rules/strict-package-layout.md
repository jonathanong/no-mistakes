# `strict-package-layout`

Enforces configured package directory layout.

```yaml
rules:
  - rule: strict-package-layout
    scope: repository
    options:
      testFilePatterns: ["*.test.ts", "*.spec.ts"]
      testDirName: "__tests__"
      packages:
        - root: packages
          sourceExtension: .ts
          allowedRootFiles: [package.json, README.md, index.ts]
          allowedSubdirs: [src, __tests__]
```

Counterexample: a package missing required files or containing banned paths.

Fix: add/move files to the configured package layout.
