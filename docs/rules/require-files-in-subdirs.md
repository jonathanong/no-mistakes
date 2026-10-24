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
and at least one file from each `requireAnyOf` group. Each `root` is
normalized before the scan: `./`, empty segments, and in-repo `..` collapse,
so `services/../packages` is `packages`. A root that leaves the repository,
such as `..`, is one finding on the config file at line 1
(`..: configured root escapes the repository`). The parent directory is not
scanned.

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

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it checks file presence in the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Use the rule application's include/exclude scope for intentional exceptions;
use a file directive only when a single selected file is the policy boundary.

## Related rules

[`required-local-docs`](required-local-docs.md) enforces colocated docs, while
[`strict-package-layout`](strict-package-layout.md) validates package shape.
