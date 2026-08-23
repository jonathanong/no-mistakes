# `csharp-no-async-void-delegate`

Flags C# `async` lambdas passed to void `Action` APIs. The default constructors
are `Command` (including `Microsoft.Maui.Controls.Command` and generic
`Command<T>`). The default method is `BeginInvokeOnMainThread`.

Why: an async lambda on a void `Action` becomes an unobserved fire-and-forget
task, so exceptions never reach the caller.

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

Options:

- `constructors` — type names after `new` (default `["Command"]`)
- `methods` — callee names (default `["BeginInvokeOnMainThread"]`)
- `allow` — path globs to skip
- `message` — extra hint appended to the finding
