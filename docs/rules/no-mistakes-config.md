# `no-mistakes-config`

## Why and when

Enable this rule in every configured repository: stale paths and ineffective
globs otherwise make a policy appear active while silently selecting nothing.

## What it catches

It validates configuration path references, scoped project globs, full-suite
trigger paths, empty ignore/exclude patterns, and incompatible `limit` plus
`direct` groups.

## Options

There are no rule-specific options or defaults. It validates the active
`.no-mistakes.yml`; generic application fields only decide where config loads.

## Valid example

The fixed configuration below uses a tracked project root, removes the empty
exclude, and puts the budget on a non-`direct` group.

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
