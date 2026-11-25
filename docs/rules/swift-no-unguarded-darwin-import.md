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

The rule reports explicit `Darwin` imports in selected `.swift` files on the
line of the import. It covers plain and selective imports, including an access
modifier or argument-free attributes on the import line. It ignores matching
text in comments, strings, and extended regex literals.

The check skips imports within `#if` blocks, including alternate branches.
Swift evaluates the condition. Use `#if canImport(Darwin)` to check the
module's availability. Scope the rule with `include` and `exclude` so
Apple-only targets can keep direct imports.

## Options and defaults

The rule is off by default. `options.allow` defaults to an empty set of path
globs. The optional `options.message` appends a hint to the default fix.
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

The check reads explicit import lines and conditional compilation markers. It
does not resolve module aliases, interpret platform predicates, or infer test
paths. Swift files with invalid UTF-8 are decoded lossily. Unreadable files
are skipped.

## Suppression

Use `// no-mistakes-disable-next-line swift-no-unguarded-darwin-import -- platform-selected target`
above the import. A `no-mistakes-disable-line` directive can go after the
import, and a `no-mistakes-disable-file` directive can cover the file. Use
`options.allow` for a stable path exception.

## Related rules

- [`swift-viewmodel-main-actor`](swift-viewmodel-main-actor.md) checks actor
  isolation on Swift view models.
- [`swift-no-raw-print`](swift-no-raw-print.md) checks raw Swift print calls.
