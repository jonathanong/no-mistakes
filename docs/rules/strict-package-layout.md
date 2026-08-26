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

## Why and when

Use this rule when every workspace package should expose a small, predictable
surface and keep tests, source, and metadata in known directories.

## What it catches/requires

Each configured package must contain the required root files and keep source
files and subdirectories within the declared layout. Test patterns identify
test files for the test-directory policy.

## Options and defaults

Each `packages` entry supplies `root`, `sourceExtension`, `allowedRootFiles`,
`allowedSubdirs`, `testFilePatterns`, and `testDirName`; values are policy
inputs rather than inferred defaults. Unlisted files or directories are
findings when the corresponding allowlist is present.

## Valid example

```text
packages/api/package.json
packages/api/README.md
packages/api/src/index.ts
packages/api/__tests__/index.test.ts
```

## Counterexample

```text
packages/api/tmp/debug.ts
packages/api/index.js
```

These paths violate the configured root/subdirectory and extension policy.

## Fix

Move files into an allowed directory, add an allowed root file only when it is
part of the package contract, or adjust the package policy deliberately.

## Suppression

Prefer a package-specific configuration entry for generated or legacy packages.
Use a file directive only for a single externally managed artifact.

## Related rules

[`require-files-in-subdirs`](require-files-in-subdirs.md) requires named files;
[`production-dependency-declarations`](production-dependency-declarations.md)
checks runtime dependency fields.
