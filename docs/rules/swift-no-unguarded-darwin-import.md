# `swift-no-unguarded-darwin-import`

<!-- cspell:ignore getpagesize macCatalyst -->

## Why and when

Enable this rule on portable SwiftPM test targets that also compile on Linux. An
unconditional import of an Apple-only module prevents the test target from
compiling where the module is unavailable.
[Swift conditional compilation](https://docs.swift.org/latest/documentation/the-swift-programming-language/statements/#Conditional-Compilation-Block).

## Configuration

```yaml
rules:
  - rule: swift-no-unguarded-darwin-import
    scope: repository
```

## What it catches

The rule inspects `.swift` files that belong to declared `.testTarget(...)`
entries in a portable `Package.swift`. It skips:

- Non-test SwiftPM targets (`.target`, `.executableTarget`, `.macro`, `.plugin`,
  and `.systemLibrary`), such as a macOS app's `Sources/` entry point.
- Xcode and XcodeGen app projects that have no `Package.swift`.

A `platforms:` list only sets deployment floors, so it never exempts a package.

When an enclosing `Package.swift` is unreadable or its `platforms:` or
`targets:` cannot be determined statically (for example, conditional compilation
in the manifest or dynamic helper expressions), matching imports are routed
through `options.unanalyzable` instead of reported as definite findings.

Inside portable SwiftPM test targets, the rule reports plain and selective
imports of configured modules. It points to the `import` keyword when the
declaration spans lines, follows a semicolon, or has attributes with arguments.
Comments and import-shaped text in strings and regex literals are ignored.

A `canImport(Module)` guard proves an import of that same module. An `os(...)`
guard proves the branch excludes Linux when it names a recognized Apple or
non-Linux platform (`macOS`, `iOS`, `tvOS`, `watchOS`, `visionOS`, `Windows`,
`FreeBSD`, and similar). Conjunctions (`&&`) combining a proven guard with
other predicates—such as `#if os(iOS) && !targetEnvironment(macCatalyst)` or
`#if os(macOS) && canImport(AppKit)`—and disjunctions (`||`) where every branch
proves the guard—such as `#if (os(iOS) || os(macOS)) && !targetEnvironment(macCatalyst)`—are
decidable. These proofs apply to the selected `#if` or `#elseif` branch and its
nested blocks. Each sibling branch needs its own proof. A branch without a
provable guard is unanalyzable. This includes bare feature flags (`#if DEBUG`),
negated guards (`#if !os(macOS)`), disjunctions with an unproven branch
(`#if canImport(Darwin) || DEBUG`), and `#else` without a proven enclosing
guard.

## Options and defaults

The rule is off by default. `options.modules` defaults to `["Darwin"]`. Set it
to replace that list with the Apple-only modules your repository wants to guard.

Recommended opt-in list for portable SwiftPM test targets:

```yaml
rules:
  - rule: swift-no-unguarded-darwin-import
    scope: repository
    options:
      modules: [Darwin, Security, UIKit, AppKit]
```

Use bare, case-sensitive module identifiers without spaces, periods, or
backticks. An empty list selects no modules. An invalid entry fails the check
with exit code 2. `CoreFoundation` is available on Linux and is omitted from the
recommended list.

`options.allow` defaults to an empty set of path globs. The optional
`options.message` appends a hint to the default fix.
Use `options.unanalyzable` to route unproven guards, manifests whose targets or
platforms cannot be determined statically, and unreadable files.
It defaults to `report`. Use `advise` for non-failing advisories or `ignore`
to omit these diagnostics. Unconditional imports in proven portable test targets
remain findings.
The CLI and the async Node `check()`, `checkJson()`, and `analyzeProject()`
check report use the same rule.

## Valid example

```swift
// Package.swift
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Clock",
    targets: [
        .testTarget(name: "ClockTests"),
    ]
)
```

```swift
// Tests/ClockTests/ClockTests.swift
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
// Package.swift
// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Clock",
    targets: [
        .testTarget(name: "ClockTests"),
    ]
)
```

```swift
// Tests/ClockTests/ClockTests.swift
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
guard, an Apple-only `#if os(...)` guard, or `#if !os(Linux)`. When a test can use a portable API,
move to that API and remove the platform-specific import.

## Analysis

### Effort

Runs at `max`. An explicit `low` request emits an analysis warning and runs
the check at `max`.

### Limits

The rule parses static `Package.swift` `platforms:` and `targets:` declarations
and evaluates monotone positive Boolean combinations (`&&`, `||`, parentheses)
of `canImport(...)` and `os(...)` predicates across single-line and multiline
directives. General Swift condition evaluation and dynamic manifests are routed
through `options.unanalyzable`. Swift files with invalid UTF-8 are decoded
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
