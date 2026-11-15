---
name: native-app-rules
description: "MUST NOT / SHOULD NOT rules for native apps in Swift (iOS, macOS) and .NET (MAUI, C#): concurrency, localization, secure storage, and release config. Use when writing or reviewing Swift or C# app code."
---

# Swift and .NET app rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

## Shared

- MUST NOT hardcode user-facing copy; use typed localization keys.
- MUST NOT ship debug-only session or cookie loading in release builds, or default release endpoints to localhost.
- MUST NOT store sessions outside secure storage.
- MUST NOT fall back to a web view for a native feature without a documented blocker.

## Swift

- MUST NOT leave view models off `@MainActor`.
- MUST NOT leave Darwin-only tests ungated (`#if canImport(Darwin)`).
- MUST NOT omit the keychain access group when sandbox entitlements are on.
- SHOULD NOT call `print()` in shipped code; use a logger.

## C#

- MUST NOT pass `async` lambdas to `void` delegates such as `Action` or commands; exceptions crash unobserved. Call a `Task`-returning method that catches and reports its own errors.
