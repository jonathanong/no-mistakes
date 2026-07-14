# Performance diagnostics

Every CLI leaf inherits two opt-in diagnostics flags:

```sh
no-mistakes --timings dependencies src/app.ts --format json
no-mistakes tests plan vitest --verbose-timings --changed-file src/app.ts
```

`--timings` emits phase durations. `--verbose-timings` implies `--timings` and
also emits deterministic work counts for discovery, source reads, parsing,
resolution, graph construction, traversal, and output. Diagnostics go only to
stderr; stdout, structured output order, exit status, and actionable error text
are unchanged.

```text
[timing] graph: 12.411ms
[timing] analysis.rules: 7.034ms (parallel; non-additive)
[timing] total: 21.882ms
[work] discovery.roots: 1
[work] parse.files: 42
[work] source.reads: 42
```

Parallel phase durations can overlap their enclosing phase and siblings. Never
add lines marked `parallel; non-additive` to estimate total time; use `total`
for wall time.

Instrumentation is invocation-scoped and disabled by default. A disabled run
does not construct an observer, start clocks, or retain work ledgers. Enabled
runs memoize successful and failed discovery/read/parse operations in memory
for the duration of that invocation only.

The async Node API remains stderr-free. `impactedChecks({ timings: true })`
retains its structured `{ phase, duration_ms }[]` response; other Node methods
do not add timing fields.
