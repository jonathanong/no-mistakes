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

## Why and when

Use this rule for package or service directories that must be self-describing
and expose a predictable entrypoint contract.

## What it catches/requires

Each directory selected by `packages` must contain every `requiredFiles` entry
and at least one file from each `requireAnyOf` group.

## Options and defaults

`packages` is required; each item has `root`, `requiredFiles`, and optional
`requireAnyOf`. There are no implicit roots or required filenames.

## Valid example

```text
packages/api/README.md
packages/api/package.json
packages/api/src/index.ts
```

## Counterexample

```text
packages/api/package.json
```

The directory is missing its required README and entrypoint alternative.

## Fix

Add the missing files or narrow the package root so generated/vendor folders
are not selected.

## Suppression

Use the rule application's include/exclude scope for intentional exceptions;
use a file directive only when a single selected file is the policy boundary.

## Related rules

[`required-local-docs`](required-local-docs.md) enforces colocated docs, while
[`strict-package-layout`](strict-package-layout.md) validates package shape.
