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
  "fixtures": { "changed": ["account.profile"], "added": [], "removed": [] },
  "unclassified": [],
  "complete": true
}
```

Supported keyed formats are pnpm/npm lockfiles, SwiftPM `Package.resolved`
versions 1–3, NuGet `packages.lock.json` per target framework, JSON objects or
keyed arrays, Apple `.strings` (UTF-8 or BOM-marked UTF-16), and .NET `.resx`.
The localization format is old-style `.strings`, not `.xcstrings`.

Category `exclude` globs apply only to the category that declares them; another
category whose `paths` match the same path still classifies and diffs it. A
changed file matched by at least one category's `paths` that every matching
category excludes, or that no rule in a matching category can classify (such as
a changed symbolic link matching a `keysOf` category), appears in
`unclassified`. Changed files matched by no category's `paths` (such as ordinary
source files) are recorded in `changedFiles` (omitted when empty) so
`tests plan --changes` can route them through the import graph on its own
without requiring `--changed-file` or `--base`. Unreadable roots or matched files, malformed
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
`ChangesReport`, `KeyChanges`, and `ChangesConfig` for the public contracts.
