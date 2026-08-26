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

## Why and when

Use this rule for Swift UI code where ViewModels can be called from arbitrary
tasks and must serialize UI-state mutation on the main actor.

## What it catches/requires

Class declarations whose names end in the configured suffix must carry the
configured actor attribute. Structs, actors, and extensions are ignored.

## Options and defaults

`suffix` defaults to `ViewModel`; `attribute` defaults to `MainActor`;
`allow` defaults to empty path globs; `message` is optional. Rule include/exclude
filters select Swift files.

## Valid example

```swift
@MainActor
final class ProfileViewModel {}
```

## Counterexample

```swift
final class ProfileViewModel {}
```

## Fix

Annotate the class with `@MainActor` (or the explicitly accepted unsafe form)
and keep UI mutations on that actor.

## Suppression

Use `allow` for a stable generated or framework-owned path. Use a line or file
`no-mistakes-disable-* swift-viewmodel-main-actor` directive only with a reason.

## Related rules

[`swift-no-raw-print`](swift-no-raw-print.md) covers Swift runtime hygiene;
Swift test planning and package boundaries are documented in the configuration
guide.
