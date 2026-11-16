---
name: native-app-rules
description: "Load before writing or reviewing Swift or SwiftUI (iOS, macOS) or C# and .NET MAUI app code. Lists the defects to avoid and the patterns that need a stated reason: core-library boundaries, concurrency, localization, secure storage, release config."
---

# Swift and .NET app rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Shared

- MUST NOT hardcode user-facing copy; use typed localization keys.
- MUST NOT ship debug-only session or cookie loading in release builds, or default release endpoints to localhost.
- MUST NOT store sessions outside secure storage.

## Swift

- MUST NOT leave view models off `@MainActor`.
- MUST NOT leave Darwin-only tests ungated (`#if canImport(Darwin)`).
- MUST NOT omit the keychain access group when sandbox entitlements are on.
- SHOULD NOT call `print()` in shipped code; use a logger.

## C#

- MUST NOT pass `async` lambdas to `void` delegates such as `Action` or commands; exceptions crash unobserved. Call a `Task`-returning method that catches and reports its own errors.
