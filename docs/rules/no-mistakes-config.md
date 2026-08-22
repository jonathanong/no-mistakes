# `no-mistakes-config`

Lints the loaded `.no-mistakes.yml` against tracked files: schema path
fields must exist, ignore/exclude globs must match something, and an
environment-level `limit` must not share a budget with a `direct` group
(issue #9440).

```yaml
rules:
  - rule: no-mistakes-config
    scope: repository
```

Counterexample: `projects.web.root` points at a directory that is not in
the tree, an `exclude` glob matches nothing, or `prePush` sets `limit`
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
