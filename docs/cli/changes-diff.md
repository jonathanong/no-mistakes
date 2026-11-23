<!-- cspell:ignore resx xcstrings -->
# `no-mistakes changes diff`

Compare two local directory trees using configured categories and semantic IDs.
The caller fetches or prepares the directories first. This command does not
invoke version control, contact a server, or infer repository history.

```sh
no-mistakes changes diff --base-root before --head-root after --config changes.yml
no-mistakes tests plan vitest --root after --changes changes.json --format json
```

Redirect the first command's JSON to `changes.json` before running the second.
Relative `--config` paths are relative to `--head-root`. Without it, the ordinary
configuration loader uses the head tree. See [changes configuration](../configuration/changes.md).

Each declared category produces sorted `changed`, `added`, and `removed` IDs.
A path-only category reports relative paths. `id.fromPath` extracts IDs from a
path template; `id.keysOf` compares structured contents. Multiple rules and
locale files combine their IDs within a category. Reordering keys or entries
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

Unclassified changed paths appear in `unclassified`; configured excludes do not
appear in any category or in that list. Unreadable roots or files, malformed
structured files, and unsupported entries produce `complete: false` with
`reasons`. An unreadable root still produces valid report JSON and exits 1.
Complete reports exit 0, even with unclassified paths; the test planner treats
any unclassified path as requiring full-suite selection.

The async Node equivalent is `changesDiff({ baseRoot, headRoot, config })`.
Its promise resolves to the same report, including incomplete results. Argument
or configuration errors reject instead. Import `ChangesDiffOptions`,
`ChangesReport`, `KeyChanges`, and `ChangesConfig` for the public contracts.
