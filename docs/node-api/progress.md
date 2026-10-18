<!-- cspell:ignore THREADPOOL -->

# Live phase progress

Repository-analysis APIs accept a nonempty `diagnosticsId` to opt into live,
request-scoped phase progress. Poll asynchronously while that promise is pending:

```js
const pending = check({ root: ".", diagnosticsId: "repo-check" });
const snapshot = await diagnosticsSnapshot({ id: "repo-check" });
// snapshot?.active: [{ id, phase, startedMs, elapsedMs, parallel }]
// snapshot?.completed: [{ label, durationMs, parallel }]
await pending;
// diagnosticsSnapshot({ id: "repo-check" }) now returns null.
```

The exported `DiagnosticsProgressSnapshot`, `ActiveTimingSpan`, and
`CompletedTimingSpan` types describe the snapshot. `elapsedMs` is request wall
time; `startedSpans` and `completedSpans` count instrumented trace spans.
Parallel and nested durations overlap. Snapshot polling uses the native async
worker separate from the libuv analysis pool, without waiting for the filesystem
invocation lock. Polling works with `UV_THREADPOOL_SIZE=1` and with all analysis
workers waiting on the lock. Its bounded queue rejects overload as a promise.
A snapshot can be `null` before the worker registers the request. Active IDs must be unique; completion, rejection, and unwinding
remove the registry entry. No completed payload is retained across requests.

Polling does not cancel analysis, write stderr, or install process signal
handlers. Callers choose their polling interval and cancellation policy. The
CLI counterpart is [`--progress`](../cli/diagnostics.md), with an opt-in flushed
heartbeat and a Unix termination snapshot.

The async `effects()` API shares the CLI receiver resolution.

Synchronous inline arrows, function expressions, and returned helpers retain their enclosing
invocation context when resolving supplied receiver arguments and defaults. A
later receiver write in that same context does not invalidate an earlier direct
helper call. Calls after a write and captured closures with uncertain execution
order remain unresolved; synthetic reachability edges do not add a second
unknown argument set to a proven invocation. Returned helpers require every
known factory invocation to call its result immediately; a stored result or
opaque escape keeps receiver arguments unresolved. Async and generator helpers
remain conservative because their receiver reads may occur after suspension.

Transparent aliases of default receiver parameters preserve the eventual member
read. For example, `const alias = client; alias.get()` retains the original
`get` target after only a sibling `set` write. Matching and dynamic writes still
invalidate live members. A copied method such as `const get = alias.get` keeps
its acquisition snapshot; copying a container keeps descendant members live.
For a direct, module-level arrow invocation executed once, a copied method
also keeps its original target when that same body writes the member afterward.
This includes an IIFE evaluated in `export default`, directly or as an object
member value. Exported functions and object methods retain their callable guards.
This includes classic `for` initializers and `for-in`/`for-of` right-hand
expressions, which execute once per loop statement. Tests, updates, bodies,
per-item bindings, and sloppy-script Annex B initializers remain conservative.
Setup nested in an outer loop, repeatable function, or instance initializer also
remains unresolved when a later execution could copy a replacement.
Captured and suspended helpers remain conservative. Nested propagation without
an exact assigned member history also retains the conservative default guard.

`testsTargets({ framework: "cargo", ... })` and `testsPlan()` use visible Cargo
manifests for package names and target selectors, including visible exact paths outside their declaring package directory.
They do not execute Cargo metadata. Rust unit-test modules select an
unambiguous enabled library/binary target; ambiguous helpers, missing manifests,
disabled targets, and unresolved required features retain empty `targets` and
`cargo-target-unresolved` warnings. `analyzeProject()` preserves the same plan
and warnings. Inspect these warnings before running the structured argv;
package-wide source fallback requires explicit `tests.rust.packages` config.

Visible exact Cargo target entries outside their declaring package directory
retain that declaring package and selector. When several selected packages
declare the same entry, all enabled targets are returned deterministically.
Configured package scopes still apply to the declaring packages; external
entries do not make neighboring helpers or other packages eligible.
The same prepared receiver facts track supported replacement RHS aliases for
`effects`, call-graph reports, and checks. After `box.wrapper = retained`, a
matching descendant write reaches the receiver held by `retained`; restored
aliases, saved containers, and copied methods retain their acquisition order.
Literal RHS object and array values project into mutable bare-root assignment
patterns. Unknown and conditional stores keep possible origins. These local
projections do not infer arbitrary heap transfers through function calls.

[Node/N-API guide](../node-api.md)
