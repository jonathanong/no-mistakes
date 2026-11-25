# `swift-no-unguarded-darwin-import`

<!-- cspell:ignore getpagesize -->

## Why and when

Enable this rule on Swift test sources that also compile on Linux. An
unconditional import of an Apple-only module prevents the test target from
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

The rule reports plain and selective imports of configured modules in selected
`.swift` files. It points to the `import` keyword when the
declaration spans lines, follows a semicolon, or has attributes with arguments.
Comments and import-shaped text in strings and regex literals are ignored.

A direct `canImport(Module)` guard proves an import of that same module.
A direct `os(...)` guard proves the branch excludes Linux when it names a
recognized non-Linux platform, such as `macOS`, `Windows`, or `FreeBSD`.
These proofs apply to the selected `#if` or `#elseif` branch and its nested
blocks. Each sibling branch needs its own proof. A branch without a provable
guard is unanalyzable.
This includes feature flags, negated or compound conditions, and `#else`
without a proven enclosing guard.

Scope the rule with `include` and `exclude` so Apple-only targets can keep
direct imports.

## Options and defaults

The rule is off by default. `options.modules` defaults to
`[Darwin, Security, UIKit, AppKit]`. Set it to replace that list with the modules
your repository needs to guard.

```yaml
rules:
  - rule: swift-no-unguarded-darwin-import
    scope: repository
    include: ["Tests/**"]
    options:
      modules: [Darwin, Security, SampleFramework]
```

Use bare, case-sensitive module identifiers without spaces, periods, or
backticks. An empty list selects no modules. An invalid entry fails the check
with exit code 2. `CoreFoundation` is available on Linux and is omitted from the
default list.

`options.allow` defaults to an empty set of path globs. The optional
`options.message` appends a hint to the default fix.
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

Wrap the import and the tests that use it in a matching `#if canImport(...)`
guard. When a test can use a portable API, move to that API and remove the
platform-specific import.

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
