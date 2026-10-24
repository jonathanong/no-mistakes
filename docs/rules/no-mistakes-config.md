# `no-mistakes-config`

## Why and when

Enable this rule in every configured repository: stale paths and ineffective
globs otherwise make a policy appear active while silently selecting nothing.

## What it catches

It validates configuration path references, scoped project globs, full-suite
trigger paths, empty ignore/exclude patterns, and incompatible `limit` plus
`direct` groups.

`tests.playwright.navigationHelpers` contains bare or dotted callee names,
not filesystem paths. Names such as `navigateTo` and `navigation.navigateTo`
do not require corresponding files.

## Options

There are no rule-specific options or defaults. It validates the active
`.no-mistakes.yml`; generic application fields only decide where config loads.

## Valid example

The project root below exists in the example tree.

```yaml
# .no-mistakes.yml
projects:
  web:
    root: web
rules:
  - rule: no-mistakes-config
    scope: repository
```

```ts
// web/index.ts
export const appName = 'web';
```

## Counterexample

```yaml
# .no-mistakes.yml
projects:
  web:
    root: apps/missing
rules:
  - rule: no-mistakes-config
    scope: repository
```

The configured project root does not exist, so the rule reports `apps/missing`.

## Unknown-key limitation

Unknown top-level keys are accepted by the current loader and are not findings
of this rule. The following active configuration loads successfully and reports
nothing; `unknownKey` has no effect. This is a limitation, not a reporting
counterexample:

```yaml
# .no-mistakes.yml
unknownKey: true
rules:
  - rule: no-mistakes-config
    scope: repository
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it validates the parsed no-mistakes configuration, so `options.unanalyzable` has no effect on it.

## Suppression

Fix configuration rather than suppressing it. For an intentional generated
config, use a narrowly scoped file directive in a comment-capable source.

## Related rules

[`config-path-references`](config-path-references.md) checks other structured
config paths; [`structured-config-policy`](structured-config-policy.md) checks
their shape.

Lints the loaded `.no-mistakes.yml` against tracked files: schema path
fields must exist, project `include`/`exclude` globs are resolved relative to
their project root, positive full-suite trigger paths must match,
ignore/exclude globs must match something, and an
environment-level `limit` must not share a budget with a `direct` group
(issue #9440).

```yaml
rules:
  - rule: no-mistakes-config
    scope: repository
```

Counterexample: `projects.web.root` points at a directory that is not in
the tree, `projects.web.include` points outside the project root, a positive
full-suite trigger path is missing, an `exclude` glob matches nothing, or
`prePush` sets `limit`
while also listing a `direct` group.

```yaml
projects:
  web:
    root: apps/missing
testPlan:
  vitest:
    environments:
      prePush:
        limit:
          files: 10
        exclude: ["gone/**"]
        groups:
          - type: direct
```

Fix: point path fields at tracked files or directories, delete empty
globs, and move the budget onto non-`direct` groups so changed tests are
not dropped.

```yaml
projects:
  web:
    root: web
testPlan:
  vitest:
    environments:
      prePush:
        groups:
          - type: direct
          - type: dependencies
            limit:
              files: 10
```

## Limitations

YAML 1.1 merge keys (`<<: *anchor`) are not expanded when the config loader
reads `.no-mistakes.yml`. `serde_yaml` leaves `<<` as a key, and the project
mapping ignores that key. Schema path fields that the anchor would have
supplied stay unset, so this rule never checks them against tracked files.
Project `include` and `exclude` globs are the same kind of path field: they
are resolved only after the loader has copied them onto the project.

```yaml
shared: &shared
  root: apps/missing
projects:
  web:
    <<: *shared
```

That file loads successfully. `projects.web.root` stays unset, and
`apps/missing` is not reported as a missing directory.

A bare alias is not this limit. `root: *alias` is expanded by the YAML parser,
so the loaded path is checked like any other `root`.
