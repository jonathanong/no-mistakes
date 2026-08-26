# `csharp-no-async-void-delegate`

Flags C# `async` lambdas passed to void `Action` APIs. The default constructors
are `Command` (including `Microsoft.Maui.Controls.Command` and generic
`Command<T>`). The default method is `BeginInvokeOnMainThread`.

## Why and when

An async lambda on a void `Action` becomes an unobserved fire-and-forget
task, so exceptions never reach the caller.

## What it catches

The rule recognizes configured constructor and method call names when their
delegate argument is an `async` lambda that will be converted to `void Action`.

## Valid example

`new Command(() => _ = RefreshAsync())` and
`BeginInvokeOnMainThread(() => _ = RefreshAsync())` keep the delegate itself
synchronous and pass.

```yaml
rules:
  - rule: csharp-no-async-void-delegate
    scope: repository
    include:
      - "src/App/**/*.cs"
```

Counterexample: `new Command(async () => await RefreshAsync())` or
`BeginInvokeOnMainThread(async () => await RefreshAsync())`.

Fix: wrap the work as `() => _ = FooAsync()` so the delegate stays synchronous.

## Options and defaults

- `constructors` — type names after `new` (default `["Command"]`)
- `methods` — callee names (default `["BeginInvokeOnMainThread"]`)
- `allow` — path globs to skip
- `message` — extra hint appended to the finding

## Suppression

Use `no-mistakes-disable-next-line csharp-no-async-void-delegate` on the lambda
or `no-mistakes-disable-file` for generated code. Prefer a synchronous wrapper
because suppression leaves unobserved exceptions possible.

## Related rules

[`csharp-max-lines-per-file`](csharp-max-lines-per-file.md) is the companion
C# maintainability check; it does not alter async delegate behavior.
