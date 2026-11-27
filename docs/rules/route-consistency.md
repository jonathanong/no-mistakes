# `route-consistency`

Configures which files become `route` edges: a frontend reference to a backend route definition, a project server route, or a page file under `frontendRoot`.

## Why and when

Use this rule when graph queries should connect a literal route reference to the file that defines that route. It does not emit a `check` finding. Empty options, aside from discovered project server routes, produce no route edges.

## What it requires

Route edges are collected only when at least one of these is true: the project has server-route globs, both `backendPattern` and `backendRegisterObject` are non-empty, or `frontendRoot` is non-empty. Backend definitions come from files matching `backendPattern` that register a literal path on `backendRegisterObject`. An invalid `backendPattern` contributes no backend definitions. `frontendRoot` adds page files under that directory. Configured rewrites add virtual definitions for those same patterns.

`scanPatterns` selects the files whose route references are walked. When `backendPrefixes` or `backendExactPaths` is non-empty and `frontendRoot` is empty, a referenced pattern is kept only when it starts with a prefix or equals an exact path. With `frontendRoot` set, that filter does not drop references.

`http-route-static-paths` and `http-call-static-paths` do not replace these fields for `route` edges. They override the same names only while HTTP edges are built.

## Options

Every field defaults to empty. Unknown option keys fail configuration validation. An empty string or empty list leaves that input out. An invalid glob is ignored.

```yaml
rules:
  - rule: route-consistency
    scope: repository
    options:
      backendPattern: backend/api/v1/**/*.mts
      backendRegisterObject: app
      frontendRoot: web/app
      scanPatterns:
        - web/**/*.tsx
      backendPrefixes:
        - /api/
      backendExactPaths: []
```

| Option | Default | Effect |
| --- | --- | --- |
| `backendPattern` | `""` | Glob of files that register backend routes. Empty or invalid installs no backend extractor. |
| `backendRegisterObject` | `""` | Binding that receives literal `.get`, `.post`, and the other verb calls. |
| `frontendRoot` | `""` | Directory whose page files are route definitions. Also disables the prefix filter. |
| `scanPatterns` | `[]` | Reference-file globs. Empty uses `**/*.tsx`, `**/*.ts`, and `**/*.mts`. Invalid entries are dropped. |
| `backendPrefixes` | `[]` | Path prefixes that count as backend references when `frontendRoot` is empty. |
| `backendExactPaths` | `[]` | Exact referenced paths that count as backend references when `frontendRoot` is empty. |

## Valid example

The register object and the reference both use the same literal path.

```ts
app.get("/api/users/:id", handler);
```

```tsx
const router = useRouter();
router.push("/api/users/42");
```

`router` must be the `useRouter()` binding. An `<a href="/api/users/42">` literal is also a reference. With `backendPrefixes: ["/api/"]` and an empty `frontendRoot`, `/api/users/42` is kept and can match `/api/users/:id`.

## Counterexample

A computed path is not a route definition and not a route reference, so the graph records no `route` edge.

```ts
app.get(path, handler);
```

```tsx
const router = useRouter();
router.push("/api/" + id);
```

A prefix filter with an empty `frontendRoot` also drops a literal reference that matches neither `backendPrefixes` nor `backendExactPaths`.

## Fix

Register and refer to the route with a string literal on the configured object, and point `backendPattern` at the file that contains the registration. Add a prefix or exact path when the reference should survive the backend filter, or set `frontendRoot` when page files are the definitions.

## Suppression

This rule does not emit a line finding. A `no-mistakes-disable-next-line` or `no-mistakes-disable-file` comment does not add a missing `route` edge or remove one the extractor recorded. Change the options or the literal path instead.

## Related rules

[`http-route-static-paths`](http-route-static-paths.md) and [`http-call-static-paths`](http-call-static-paths.md) configure `http` edges and override this rule's backend pattern, register object, and prefixes only for that extraction. Edge kinds are listed in [graph edges](../graph-edges.md).
