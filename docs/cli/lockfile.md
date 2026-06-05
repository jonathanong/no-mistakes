# lockfile diff

Show which packages changed between two lockfile versions.

## Usage

```bash
no-mistakes lockfile diff --base <ref> [--head <ref>] [--lockfile <path>] [--root <dir>] [--format json|paths]
```

## Options

| Flag | Description |
|------|-------------|
| `--base` | Base git ref to compare from (required). |
| `--head` | Head git ref (default: current working tree). |
| `--lockfile` | Path to a specific lockfile relative to `--root`. Auto-detected if omitted. |
| `--root` | Project root directory (default: current directory). |
| `--format` | Output format: `json` (default) or `paths`. |

## Supported lockfiles

| File | Manager |
|------|---------|
| `pnpm-lock.yaml` | pnpm |
| `package-lock.json`, `npm-shrinkwrap.json` | npm |
| `yarn.lock` | yarn (classic and berry) |
| `bun.lock` | bun |

Binary lockfiles (`bun.lockb`) cannot be parsed and cause a fallback warning.

## Output (json format)

Returns an array of objects, one per detected lockfile:

```json
[
  {
    "lockfile": "pnpm-lock.yaml",
    "manager": "pnpm",
    "added": ["new-package"],
    "removed": ["old-package"],
    "changed": ["lodash"]
  }
]
```

## Output (paths format)

Prints one package name per line (added + removed + changed).

## Integration with `tests plan`

When `tests plan` detects a changed lockfile and `--base` is provided, it automatically
uses this diff logic to identify impacted packages and traces them to affected tests
via `EdgeKind::PackageDependency` edges (from `package.json` → module node).

Without `--base`, providing a lockfile as `--changed-file` with `--global-config-fallback=true`
triggers a full test suite run with a `lockfile-no-baseline` warning.

## Examples

```bash
# Show what packages changed in pnpm-lock.yaml since main
no-mistakes lockfile diff --base main

# Show changed package names only (for shell scripts)
no-mistakes lockfile diff --base main --format paths

# Specific lockfile in a monorepo
no-mistakes lockfile diff --base main --root packages/api --lockfile pnpm-lock.yaml

# Plan tests impacted by lockfile changes
no-mistakes tests plan --base main --changed-file pnpm-lock.yaml
```
