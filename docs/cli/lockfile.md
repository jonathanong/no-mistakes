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
| `pnpm-lock.yaml` | pnpm, including pnpm 12 |
| `package-lock.json`, `npm-shrinkwrap.json` | npm |
| `yarn.lock` | yarn (classic and berry) |
| `bun.lock` | bun |
| `poetry.lock` | poetry |
| `uv.lock` | uv |
| `Pipfile.lock` | pipfile |
| `go.mod` | go |
| `Cargo.lock` | cargo |
| `Gemfile.lock` | gemfile |
| `composer.lock` | composer |

Auto-detection ignores `bun.lockb`. `--lockfile bun.lockb` warns that the
binary lockfile cannot be parsed, omits it, and still exits 0. In Node,
`lockfileDiff({ base, lockfile: "bun.lockb", includeWarnings: true })` returns
an empty `entries` array and a warning with `code: "binary-lockfile"`,
`lockfile`, and `message`. Without `includeWarnings`, the Node promise rejects
for explicit binary lockfiles and returns an array for supported lockfiles.

`poetry.lock`, `uv.lock`, and `Cargo.lock` are parsed as TOML. Each
`[[package]]` table contributes one package: name, version, and a fingerprint.
A `[package.dependencies]` table stays on that package. Cargo `registry+` and
`sparse+` sources are registry and use `checksum`. `git+` is git and uses
`checksum` or the source string. `path+` is a directory. A package with no
source is the workspace. Poetry git uses `resolved_reference`, then
`reference`. Other Poetry packages use the first file hash, including
`[metadata.files]`. Poetry `directory` is a directory and `url` is a tarball.
uv git uses the commit in the URL fragment after `#`, then `rev`. uv
`editable`, `directory`, and `virtual` sources are directories and may omit
`version`. A uv registry package uses the first wheel hash. `Pipfile.lock`
reads `default` and `develop`. The object key is the name. A leading `==` is
removed from `version`. A git or path entry may omit `version`; its
fingerprint is `ref` or `path`. File, URL, and other version-control sources
may omit `version`. Their source URL forms the fingerprint. A registry entry
requires `version` and uses the first hash. `go.mod` reads `require` lines, including `// indirect`.
`replace`, `exclude`, `retract`, and `go.sum`
are not packages. `Gemfile.lock` reads specs indented four spaces; deeper
lines are dependencies. A `GIT` revision and a `CHECKSUMS` `sha256` are
fingerprints, and `PATH` is a directory. `composer.lock` reads `packages` and
`packages-dev`. A null development section contributes no packages.
`dist.reference` is the fingerprint, `dist.type` `path` is a
directory, and a git source with no dist is git.

A malformed file of one of those managers is not an empty package list.
`lockfile diff` warns and omits that file. The async
`lockfileDiff()` call rejects with the same description. `pom.xml`, Gradle
lockfiles, `mix.lock`, and `pubspec.yaml` are not lockfile diffs.

pnpm 12 still writes `lockfileVersion: '9.0'`. When the project pins pnpm 12, or has config dependencies, `pnpm-lock.yaml` is two YAML documents. The env document comes first and records `configDependencies` and `packageManagerDependencies`. The project document is last. Package diffs union `packages` from every document, so a pnpm pin or config dependency change is reported. Importer and snapshot impact use the project document. A document before that project document must be an env lockfile: its `.` importer has only `configDependencies` and `packageManagerDependencies`. Anything else is rejected instead of being read as the workspace. A `resolution.revision` field is accepted metadata; the `integrity` digest remains the package identity. Lockfile majors outside 5 through 9 stay unsupported.

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

When `tests plan` detects a changed npm-family lockfile and `--base` is
provided, it automatically
uses this diff logic to identify impacted packages and traces them to affected tests
via `EdgeKind::PackageDependency` edges (from `package.json` → module node).

Without `--base`, providing a lockfile as `--changed-file` with `--global-config-fallback=true`
triggers a full test suite run with a `lockfile-no-baseline` warning. The plan
also reports `incomplete: [{ kind: "dependency-baseline-missing", files }]`.
That marker is present even when global fallback is off, so a scoped
`fallback_triggered` is not complete dependency coverage. Pass
`--base <pre-change revision>` to compare the lockfile.

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
