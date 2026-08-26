# `swift-no-raw-print`

Flags raw `print(` and `Swift.print(` calls in `.swift` files. Method calls such
as `logger.print(` are ignored. Scope files with rule `include` / `exclude`;
skip a logger fallback with `allow`.

Why: raw `print` bypasses the process logger, so debug noise and secrets cannot
be filtered or attributed in production logs.

```yaml
rules:
  - rule: swift-no-raw-print
    scope: repository
    include:
      - "clients/**/*.swift"
    options:
      allow:
        - "clients/core/Sources/Logging.swift"
```

Counterexample: `print("[DEBUG]", message())` or `Swift.print("loaded")`.

Fix: call the process logger instead of `print`.

Options:

- `allow` — path globs to skip
- `message` — extra hint appended to the finding

## Why and when

Use this rule in Swift applications and libraries where production logging must
be filterable, attributable, and safe for sensitive values.

## What it catches/requires

Raw `print(` and `Swift.print(` calls in selected `.swift` files are findings;
receiver-qualified calls such as `logger.print(` are allowed.

## Options and defaults

`allow` defaults to empty and contains path globs to skip. `message` is
optional and appends a custom hint. Rule `include`/`exclude` filters select the
Swift files.

## Valid example

```swift
logger.info("loaded profile")
```

## Counterexample

```swift
print("loaded profile")
```

## Fix

Send the event through the process logger, or add the specific logging file to
`allow` when it is the logger implementation itself.

## Suppression

Prefer `allow` for a stable logger fallback. Use
`no-mistakes-disable-next-line swift-no-raw-print` for one intentional call.

## Related rules

[`swift-viewmodel-main-actor`](swift-viewmodel-main-actor.md) protects UI
concurrency; this rule protects logging boundaries.
