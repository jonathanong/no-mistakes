# Test Plan

The `test plan` command identifies which tests are affected by code changes using dependency graph analysis.

## Input Modes

All inputs can be combined — the resulting test set is the union of all inputs.

### Explicit changed files

```bash
no-mistakes test plan --changed-file src/utils.mts --changed-file src/service.mts
no-mistakes test plan --changed-files changed-files.txt
```

### Git diff (explicit)

```bash
no-mistakes test plan --base main
no-mistakes test plan --base main --head feature-branch
```

### Unified diff

```bash
no-mistakes test plan --diff path/to/changes.diff
no-mistakes test plan --diff-stdin < <(git diff main)
no-mistakes test plan --diff-command "git diff main"
```

### Entrypoints (file#export)

```bash
no-mistakes test plan --entrypoint "src/utils.mts#formatDate"
no-mistakes test plan --entrypoint "src/a.mts" --entrypoint "src/b.mts#handler"
```

## `test impact` Command

Convenience subcommand for entrypoint-based impact analysis:

```bash
no-mistakes test impact "src/utils.mts#formatDate" "src/service.mts"
```

Equivalent to:

```bash
no-mistakes test plan --entrypoint "src/utils.mts#formatDate" --entrypoint "src/service.mts"
```

## Output Formats

```bash
no-mistakes test plan --diff-command "git diff main" --json          # JSON (default)
no-mistakes test plan --diff-command "git diff main" --format paths  # one test file per line
no-mistakes test plan --diff-command "git diff main" --format md     # markdown summary
```

## Deleted File Handling

When a diff indicates a file was deleted, the tool:

1. Adds the deleted file as a phantom node in the dependency graph
2. Finds all files that reference the deleted file (broken imports)
3. Traces from those files to find affected tests
4. Emits a `deleted-file` warning in the output

## Programmatic API (N-API)

```javascript
const { testsPlan, testsImpact } = require("no-mistakes");

// Diff-based
const plan = await testsPlan({
  root: "/path/to/project",
  diff: unifiedDiffString,
});

// Entrypoint-based
const plan = await testsImpact({
  root: "/path/to/project",
  entrypoints: ["src/utils.mts#formatDate", "src/service.mts"],
});
```

## Lockfile Change Handling

When a lockfile (`pnpm-lock.yaml`, `package-lock.json`, `yarn.lock`, `bun.lock`) appears
in the changed file list, `tests plan` performs targeted package-level analysis instead of
falling back to the full test suite. This applies to both plain plans and framework
(Playwright / Vitest) configured plans.

1. Parse the old lockfile version (from `--base` git ref) and the new working-tree version.
2. Diff the two to find added, removed, and changed package names.
3. Trace from each changed package name (`NodeId::Module`) through `PackageDependency` and
   import edges in the dependency graph to reach affected test files. For workspace packages
   the graph records a `NodeId::File(entry)` instead; the workspace map is consulted as a
   fallback when no `Module` node is present.

For framework (Playwright/Vitest/Swift) plans the BFS-found tests are injected into the
`dependencies` group, exactly mirroring the non-framework path.

Full-suite fallback fires only for:

- **Unparsable lockfiles**: diff-only mode without `--head`, binary lockfiles (`bun.lockb`).
  These are unconditional (do not require `--global-config-fallback`).
- **Genuinely untraceable deps**: tooling packages (`typescript`, `eslint`, etc.) that have
  no import-graph path to any test file. These fall back only when
  `--global-config-fallback=true` is set (or `globalConfigFallback: true` in the
  environment config).

This requires `--base` (or another mechanism to supply the old lockfile content).
Without `--base`, a `lockfile-no-baseline` warning is emitted and `--global-config-fallback=true`
triggers a full suite run.

Binary lockfiles (`bun.lockb`) cannot be parsed and always trigger a warning + fallback.

```bash
# Targeted: only tests affected by lodash version bump run (plain plan)
no-mistakes test plan --changed-file pnpm-lock.yaml --base main

# Targeted: same behavior for Playwright framework plan
no-mistakes test plan playwright --changed-file pnpm-lock.yaml --base main

# Full suite fallback (no baseline supplied)
no-mistakes test plan --changed-file pnpm-lock.yaml --global-config-fallback=true

# Tooling dep bump → fallback only when flag is set
no-mistakes test plan playwright --changed-file pnpm-lock.yaml --base main --global-config-fallback=true
```

## Breaking Change: Implicit Git Removed

Previously, running `no-mistakes test plan` with no input arguments would implicitly run
`git diff`, `git diff --cached`, and `git ls-files --others` to auto-detect changes.

This implicit behavior has been removed. Clients must now explicitly specify their input:

```bash
# Before (implicit, no longer works)
no-mistakes test plan --json

# After (explicit)
no-mistakes test plan --diff-command "git diff" --json
no-mistakes test plan --base HEAD~1 --json
```
