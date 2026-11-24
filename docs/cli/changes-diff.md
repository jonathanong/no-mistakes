<!-- cspell:ignore resx xcstrings -->
# `no-mistakes changes diff`

Compare two local directory trees using configured categories and semantic IDs.
The caller fetches or prepares the directories first. This command does not
invoke version control, contact a server, or infer repository history.

```sh
no-mistakes changes diff --base-root before --head-root after --config changes.yml > after/changes.json
no-mistakes tests plan vitest --root after --changes changes.json --format json
```

Redirect the first command's JSON to a report file before running the second.
Relative `--config` paths are relative to `--head-root`, and relative
`--changes` paths in `tests plan` are relative to `--root` (matching `--config`;
pass a parent-relative path such as `../changes.json` or an absolute path if the
report sits outside `--root`). Without `--config`, the ordinary configuration
loader uses the head tree. See [changes configuration](../configuration/changes.md).

Each declared category produces sorted `changed`, `added`, and `removed` IDs.
A path-only category reports relative paths. `id.fromPath` extracts IDs from a
path template; `id.keysOf` compares structured contents. Multiple rules and
locale files combine their IDs within a category; adding a new locale file whose
keys already exist in other locales reports those keys as `changed` rather than
`added`. Reordering keys or entries
without changing their values does not produce a change.

```json
{
  "schemaVersion": 1,
  "fixtures": {
    "changed": ["account.profile"],
    "added": [],
    "removed": [],
    "attributePaths": { "account.profile": ["/avatar/size"] }
  },
  "unclassified": [],
  "complete": true
}
```

For changed path-derived IDs with comparable JSON objects on both sides,
`attributePaths` maps each ID to sorted RFC 6901 JSON Pointers. The path names
the changed property or array. In a path segment, `~0` represents `~` and
`~1` represents `/`. If a file is added, removed, replaced, non-JSON, or
cannot be compared reliably, the ID has no attribute paths. The planner then
uses whole-fixture selection for that ID.

Supported keyed formats are pnpm/npm lockfiles, SwiftPM `Package.resolved`
versions 1–3, NuGet `packages.lock.json` per target framework, JSON objects or
keyed arrays, Apple `.strings` (UTF-8 or BOM-marked UTF-16), and .NET `.resx`.
The localization format is old-style `.strings`, not `.xcstrings`.

Category `exclude` globs apply only to the category that declares them; another
category whose `paths` match the same path still classifies and diffs it. Only
files matched by at least one category's `paths` (and not excluded by that
category) are reported; files matched by no category (such as ordinary source
files, which `tests plan` routes via `--changed-file` and the import graph) are
left out. A changed file that a category's `paths` match but no rule in it can
classify (such as a changed symbolic link matching a `keysOf` category) appears
in `unclassified`. Unreadable roots or matched files, malformed
structured files, and unsupported entries produce `complete: false` with
`reasons`. An unreadable root still produces valid report JSON and exits 1.
Symbolic links compare by link target, as git does, and are never followed:
an identical link (including a directory link or a dangling one) contributes no
change, a link with a new target or a link replaced by a regular file is a change to
the link path, and a link whose target cannot be read is reported in `reasons`
for that path only. A changed link matching a `keysOf` category is
`unclassified`, because its target is not a keyed document.
Complete reports exit 0, even with unclassified paths; the test planner treats
any unclassified path as requiring full-suite selection.

The async Node equivalent is `changesDiff({ baseRoot, headRoot, config })`.
Its promise resolves to the same report, including incomplete results. Argument
or configuration errors reject instead. Import `ChangesDiffOptions`,
`ChangesReport`, `KeyChanges`, `FixtureAttributePaths`, and `ChangesConfig` for the public contracts.
