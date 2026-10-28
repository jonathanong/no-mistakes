# Invocation Lock And Timeouts

Every async analysis function except `version()` accepts these common options:

```ts
interface InvocationOptions {
  diagnosticsId?: string;
  timeout?: number | null;
  // Opt-in positive native lock-acquisition plus execution budget; never extends timeout or lockTimeout.
  invocationTimeoutMs?: number;
  lockTimeout?: number | null;
  failOnLock?: boolean;
  jobs?: number | null;
}
```

`timeout` and `lockTimeout` use non-negative integer seconds. `timeout` limits
command execution after the lock is acquired. Without an invocation budget,
`lockTimeout` limits only the lock wait. Optional `invocationTimeoutMs` is a
positive safe integer in milliseconds when present; unlike the older options, null is invalid. It is
measured once when native invocation coordination starts, before
lock-path resolution. The lock and execution phases share this absolute budget;
it never extends either ordinary phase deadline. With this budget enabled,
`lockTimeout` starts before opening the lock file. Its deadline is compared
with the original invocation deadline, which already includes lock-path
resolution; the earlier deadline determines the timeout classification. Ordinary
lock-wait timing is unchanged when the invocation budget is omitted.
Library/Node queueing before native coordination is outside that budget. A waiting N-API request does not
change the active holder's watchdog or actor registry. `profile: "ci"` disables
this optional budget as well as the ordinary deadlines. The
Node/N-API defaults disable both deadlines (`timeout`/`lockTimeout` omitted,
`0`, or `null`). The CLI still defaults to 30 seconds; pass `0` there to
disable. `failOnLock: true` fails immediately on contention and overrides
`lockTimeout`. The first analysis request with valid UTF-8 JSON and invocation
controls that acquires the invocation lock initializes the process-wide rayon
pool before command-specific options are validated. A request rejected for a
command-specific option, such as `check({ jobs: 1, root: 123 })`, can therefore
select the pool; a later `check({ jobs: 4, root: "." })` retains that one-worker
pool. Invalid JSON or invocation controls, and requests that fail to acquire
the lock, do not select it. A positive `jobs` value selects its worker count, overriding
`RAYON_NUM_THREADS`. Pass `0` or omit it to use `RAYON_NUM_THREADS` when it is a
positive integer, otherwise the CPU count (matching CLI `--jobs 0`). Later calls retain the
initialized pool, even when they request a different `jobs` value. The call that
installs the pool does not warn. A later analysis whose resolved worker count
differs still succeeds; its JSON object includes optional `jobsPoolWarning`
(`code` `"rayon-pool-size"`, `requestedJobs`, `effectiveJobs`, and `message`)
and does not change the pool. The property is omitted when the counts match,
including when `jobs` is omitted, `0`, or `null` and that resolution equals the
recorded size. Invalid JSON or invocation controls, and requests that fail to
acquire the lock, still do not select the pool. Concurrent `analyzeProject()`
calls, and CLI runs by the same user, queue on this per-user invocation file
lock. Each queued run then uses the full recorded pool rather than a fraction
of it. PostgreSQL source batches resolve workers with the same `jobs` rules and
do not initialize the global pool before repository analysis selects it. If
that pool is already smaller than the batch's resolved count, the batch uses
that pool or another pool no larger than it, not a fresh all-CPU pool.

`import-dynamic` follows every string-literal `import()`, including `if` /
`switch` / nested-function / JSX-handler loaders (`conditional-dynamic-import`
when the static call graph does not prove they run).

Literal dynamic imports resolve relative paths, workspace wildcard `exports`,
and package `#imports` through the prepared visible resolver catalog, including
inside test callbacks and `Promise.all`. Both `import` and `import-dynamic`
follow these targets without requiring `workspace`; adding `workspace` also
records `workspace` edges. Reverse `dependents` and `related` queries
use the same edges. Computed specifiers remain unresolved and are not guessed.

Forward `dependencies` reports stay on the lazy reachable-file walk when every
requested relationship is an import kind (`import`, `import-static`,
`import-dynamic`, `import-type`, `import-require`). `dependents` and `related`
reports still prepare reverse-index facts for the visible universe. Adding
`workspace`, `test`, `route`, or any other relationship also prepares facts for
the entire visible universe. Prefer import-only relationships plus explicit
`files` when the caller only needs a module closure. `analyzeProject()` without
a `check` report no longer eagerly parses every indexable file for an
import-only `dependencies` plan.

`candidateInclude` / `candidateExclude` belong only to forward import-only
`dependencies()` / `{ type: "dependencies" }` options (`TraverseGraphOptions` /
`TraversePathsOptions`). `dependents()`, `related()`, and
`{ type: "dependents" | "related" }` take `TraverseOptions` and do not accept
those fields or `projection`. The globs define the initial candidate
inventory before GraphFiles / fact preparation. They are not `filters` and
are not merged from top-level `analyzeProject` `filters`. Unrelated excluded
files are omitted from that inventory and receive no parse/fact work on a
standalone `dependencies()` call or an exclusive same-bounds import-only
`analyzeProject` request. Mixed requests that also run `check`, `dependents`,
or unbounded graph reports keep the shared GraphFiles universe, so those
other reports may still parse the excluded files. A
resolved local or workspace source that is in the pre-filter GraphFiles
universe still escapes into the closure. `projection: "paths"` returns
`{ files: string[], diagnostics: TsConfigDiagnostic[] }` — sorted unique
repository-relative paths plus bounded diagnostics — and does not change
which paths are walked. Omit `projection` or pass `"graph"` to keep
`DependencyResult`. These candidate options are also invalid at runtime on
`includeSymbols` or non-import relationships.
`analyzeProject` report `result` stays loosely typed: a `dependencies`
report body is `ImportClosureResult` when that report sets
`projection: "paths"`, and `DependencyResult` otherwise.

`resolveCheckDependencies` derives a batch `resolveCheck` result from named
`dependencies` reports in the same request. The derived report and referenced
reports are matched by effective `root`, `tsconfig`, and `config`; report-level
scope overrides are supported, and IDs are resolved only within that scope. It
checks the union of each report's seed files and reachable local files using
the already-prepared facts, resolver catalog, and source store. `targetModules`
and folder `filters` on a
referenced report are output projections; the derived check uses the
pre-projection file closure. Referenced reports may use import and
workspace relationships, and each needs an `id`; this report does not accept
`file` or `files`.

```js
const closure = await dependencies({
  root,
  files: ["web/app/page.tsx"],
  relationships: ["import-static", "import-dynamic", "import-type"],
  candidateInclude: ["web/**"],
  candidateExclude: ["**/*.test.*"],
  projection: "paths",
});
const report = await analyzeProject({
  root,
  reports: [{
    type: "dependencies",
    files: ["web/app/page.tsx"],
    relationships: ["import-static", "import-dynamic", "import-type"],
    candidateInclude: ["web/**"],
    candidateExclude: ["**/*.test.*"],
    projection: "paths",
  }],
});
```

```js
const report = await analyzeProject({
  root,
  reports: [
    {
      type: "dependencies",
      id: "route-closure",
      files: ["web/app/page.tsx"],
      relationships: ["import-static", "import-dynamic", "import-type"],
      projection: "paths",
    },
    {
      type: "resolveCheckDependencies",
      dependencyReportIds: ["route-closure"],
    },
  ],
});
```

The lock is shared by CLI and Node/N-API analyses for the current OS user across
all repositories. While waiting, stderr reports `waiting for lock held by pid
<pid> for <n>s`. Successful return values keep their existing shapes, and lock
or timeout failures reject the returned Promise with an actionable error. For
`analyzeProject()`, put these options at the top level,
not inside individual report requests:

```js
const report = await analyzeProject({
  timeout: 60,
  lockTimeout: 10,
  failOnLock: false,
  reports: [{ type: "dependencies", files: ["src/api.mts"] }],
});
```

Public JavaScript APIs take options objects and return parsed values. Native
JSON entrypoints accept and return Node `Buffer` values of UTF-8 JSON so the
addon avoids UTF-16 string copies at the N-API boundary.

[Node/N-API guide](../node-api.md)

## Lock and deadline errors

A call that fails on the invocation lock or the analysis deadline rejects with
an `Error` that keeps the CLI's message and adds:

- `error.code`: `NO_MISTAKES_LOCK_BUSY`, `NO_MISTAKES_LOCK_TIMEOUT`, or
  `NO_MISTAKES_DEADLINE`.
- `error.details`: the same object the CLI writes under `--format json`
  (`kind`, `phase`, `message`, `exitCode`, and for lock failures `holder` with
  `pid`, `cwd`, `command`, `startedAtMs`, `ageSeconds`). Lock failures carry
  `exitCode` `75` and deadlines `124`, matching the
  [CLI exit-code contract](../cli/README.md#exit-code-contract).

Lock failures and deadlines detected at the invocation boundary are structured.
Other failures keep plain messages.

```js
try {
  await check({ root: ".", lockTimeout: 5 });
} catch (error) {
  if (error.code === "NO_MISTAKES_LOCK_TIMEOUT") {
    console.error(error.details.holder); // who holds the lock
  }
}
```

`lockStatus()` returns `{ state: "held" | "free", lockFile, holder }` without
waiting and without taking the lock; it matches `no-mistakes lock status
--format json`. The `LockStatus`, `LockHolder`, `InvocationErrorKind`,
`InvocationErrorDetails`, and `InvocationError` types are exported.
