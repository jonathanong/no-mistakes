<!-- cspell:ignore resx -->
# Categorized semantic changes

Declare opaque category names under `changes.categories`. The same configuration
controls [`changes diff`](../cli/changes-diff.md) and
[`tests plan --changes`](../cli/tests-plan.md#semantic-change-reports).

```yaml
changes:
  categories:
    fixtures:
      exclude: ["fixtures/v1/responses/generated/**"]
      rules:
        - paths: ["fixtures/v1/responses/**/*.json"]
          id:
            fromPath: "fixtures/v1/responses/{id}.json"
            separator: "."
        - paths: ["fixtures/v1/manifest.json"]
          id:
            keysOf: { format: json, array: fixtures, idField: id }
      tests: ["tests/**/*.test.ts"]
      match: 'FixtureLoader.LoadResponse("{id}")'
      fallback: fixture-tests
    localization:
      rules:
        - paths: ["App/Resources/**/*.strings"]
          id: { keysOf: strings }
        - paths: ["Messages*.resx"]
          id: { keysOf: resx }
      tests: ["tests/**/*.test.ts"]
      fallback: full-suite
```

Use `rules` alone when each path group needs its own ID rule; `paths` with `id`
is shorthand for one rule. Without an ID rule, that rule reports paths. Do not
add a path-only shorthand for the same localization files when only semantic
keys should feed the category.

`fromPath` contains one `{id}` capture. `separator` replaces slashes in the
capture, so `a/b/value.json` can become `a.b.value`. Omitting `separator` keeps
the captured text, including existing dots and slashes. `keysOf` accepts a format
name or an options object. JSON `array` is a dotted object path or JSON pointer;
`idField` names a nonempty string ID in each entry. Any field change, including
metadata, changes that entry. Duplicate IDs or unparseable data are incomplete.

The canonical formats are `pnpm`, `npm`, `swift`, `nuget`, `json`, `strings`, and
`resx`. Swift accepts Package.resolved versions 1–3. NuGet identities include the
target framework. Apple strings support comments, escapes, and UTF-16 BOMs;
comment-only or key-order changes are ignored. Resx compares only `<data name>`
entries' `<value>` children; headers and metadata entries are ignored. IDs are
combined across rules and locale variants and reported once per category.
Key presence is merged across a category's files, so adding a new locale file whose
keys already exist in other locales reports those keys as `changed` rather than
`added` (and removing a locale file whose keys still exist in other locales
reports them as `changed` rather than `removed`).

`exclude` is scoped per category: excluding a path in one category prevents only
that category's rules from matching it. Another category whose `paths` match the
path still classifies and diffs it; if every matching category excludes it, or
no rule in a matching category can classify it, the changed path goes into
`unclassified` so `tests plan --changes` keeps its safety fallback active.
Changed paths matched by no category's `paths` (such as ordinary source files)
are recorded in `changedFiles` so `tests plan --changes` routes them through
the import graph. Category names `schemaVersion`, `complete`, `unclassified`,
`changedFiles`, and `reasons` are reserved for report metadata.

`tests` limits the test sources searched for each ID. `match` is a literal
substring template containing `{id}` or `<id>`; by default it searches for the
ID itself. All changed, added, and removed IDs participate. An unmatched ID
uses `fallback`, either a named framework full-suite trigger or `full-suite`.
An absent or unusable fallback selects the full suite. Matching borrows the
prepared session's test universe and source store. For fixture attribute
selection, use the complete literal loader call as the match template, as
in `loadFixture("{id}")`. See [semantic change reports](../cli/tests-plan.md#semantic-change-reports).

To replace a broad path trigger only when a valid semantic report is supplied:

```yaml
testPlan:
  vitest:
    fullSuiteTriggers:
      triggers:
        - name: fixture-tests
          paths: ["fixtures/**"]
          supersededByChanges: true
```

Without `--changes`, trigger behavior is unchanged. Missing, invalid,
incomplete, unknown-category or unclassified input always selects the full
suite, regardless of supersession. Semantic selections are combined with the
ordinary plan; they do not remove tests selected by other analysis.
