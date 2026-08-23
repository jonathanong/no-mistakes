# `swift-viewmodel-main-actor`

Requires Swift `class` types whose names end with `ViewModel` to be annotated
`@MainActor` (including `@MainActor(unsafe)`). `struct`, `actor`, and
`extension` declarations are ignored. Scope files with rule `include` /
`exclude`.

Why: a ViewModel class without `@MainActor` can mutate UI state off the main
actor.

```yaml
rules:
  - rule: swift-viewmodel-main-actor
    scope: repository
    include:
      - "clients/**/*.swift"
```

Counterexample: `@Observable final class BrokenViewModel { var value = 0 }`.

Fix: add `@MainActor` (or `@MainActor(unsafe)`) on the class.

Options:

- `suffix` — type-name suffix (default `ViewModel`)
- `attribute` — required attribute name (default `MainActor`)
- `allow` — path globs to skip
- `message` — extra hint appended to the finding
