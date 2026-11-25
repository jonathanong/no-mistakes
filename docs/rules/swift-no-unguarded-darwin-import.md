# `swift-no-unguarded-darwin-import`

<!-- cspell:ignore getpagesize -->

## Why and when

Enable this rule on Swift test sources that compile on Apple and other
platforms. An unconditional `Darwin` import prevents the test target from
compiling where the module is unavailable.
[Swift conditional compilation](https://docs.swift.org/latest/documentation/the-swift-programming-language/statements/#Conditional-Compilation-Block).

## Configuration

```yaml
rules:
  - rule: swift-no-unguarded-darwin-import
    scope: repository
    include: ["Tests/**"]
```

## What it catches

The rule reports plain and selective `Darwin` imports in selected `.swift`
files. Other Apple-only modules such as UIKit, AppKit, and Security are
outside its scope. It points to the `import` keyword when the
declaration spans lines, follows a semicolon, or has attributes with arguments.
Comments and import-shaped text in strings and regex literals are ignored.

A direct `canImport(Darwin)` or `os(macOS)`, `os(iOS)`, `os(tvOS)`,
`os(watchOS)`, or `os(visionOS)` condition proves a guard in its `#if` or
`#elseif` branch. That proof also applies inside nested blocks. Each sibling
branch needs its own proof. A branch without a provable guard is unanalyzable.
This includes feature flags, negated or compound conditions, and `#else`
without a proven enclosing guard.

Scope the rule with `include` and `exclude` so Apple-only targets can keep
direct imports.

## Options and defaults

The rule is off by default. `options.allow` defaults to an empty set of path
globs. The optional `options.message` appends a hint to the default fix.
Use `options.unanalyzable` to route unproven guards and unreadable files.
It defaults to `report`. Use `advise` for non-failing advisories or `ignore`
to omit these diagnostics. Unconditional imports remain findings.
The CLI and the async Node `check()`, `checkJson()`, and `analyzeProject()`
check report use the same rule.

## Valid example

```swift
// Tests/ClockTests.swift
import XCTest
#if canImport(Darwin)
import Darwin

final class ClockTests: XCTestCase {
    func testPageSizeIsPositive() {
        XCTAssertGreaterThan(Darwin.getpagesize(), 0)
    }
}
#endif
```

## Counterexample

```swift
// Tests/ClockTests.swift
import XCTest
import Darwin

final class ClockTests: XCTestCase {
    func testPageSizeIsPositive() {
        XCTAssertGreaterThan(Darwin.getpagesize(), 0)
    }
}
```

## Fix

Wrap the Darwin import and the tests that use it in `#if canImport(Darwin)`.
When a test can use a portable API instead, move to that API and remove the
Darwin import.

## Analysis

### Effort

Runs at `max`. An explicit `low` request emits an analysis warning and runs
the check at `max`.

### Limits

The rule recognizes individual positive predicates, including parenthesized
forms. It leaves general condition evaluation to Swift and uses the configured
paths to identify portable tests. Swift files with invalid UTF-8 are decoded
lossily. If the source cannot be read, the diagnostic names the file and asks
the user to restore access.

## Suppression

Use `// no-mistakes-disable-next-line swift-no-unguarded-darwin-import -- platform-selected target`
above the import. A `no-mistakes-disable-line` directive can go after the
import, and a `no-mistakes-disable-file` directive can cover the file. Use
`options.allow` for a stable path exception.

## Related rules

- [`swift-viewmodel-main-actor`](swift-viewmodel-main-actor.md) checks actor
  isolation on Swift view models.
- [`swift-no-raw-print`](swift-no-raw-print.md) checks raw Swift print calls.
