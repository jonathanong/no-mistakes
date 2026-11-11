---
name: rust-rules
description: "MUST NOT / SHOULD NOT rules for Rust: lints, test layout, file size, parallelism, hot loops, caching, build hygiene, and performance claims. Use when writing or reviewing Rust code or running heavy cargo builds."
---

# Rust rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- MUST NOT silence lints with inline `#[allow]`; fix the code. Warnings fail the build.
- MUST NOT lock a global `Mutex<HashMap>` or other process-wide lock per item inside parallel iterators; shard or use concurrent maps.
- MUST NOT read files, spawn processes, compile regexes, or reparse inside a hot loop; hoist them (`LazyLock`/`OnceLock` for regexes).
- MUST NOT rebuild a repo-wide index or do linear scans per item; index once, then look up.
- MUST NOT emit nondeterministic output from parallel work; sort or merge before rendering.
- MUST NOT persist caches between runs unless the design calls for it; cache in memory per run.
- MUST NOT block an async runtime or the Node event loop with CPU-heavy native calls; run them on a thread pool.
- MUST NOT exclude files from coverage (`--ignore-filename-regex`); refactor them to meet the gate.
- MUST NOT claim a speedup without matched before/after measurements with instrumentation off.
- SHOULD NOT put tests inline in source files; keep them in separate test files.
- SHOULD NOT exceed about 200 lines per source file or 500 per test file.
- SHOULD NOT write fixture files from test code; save fixtures and copy them in.
- SHOULD NOT run several heavy cargo or coverage jobs at once on a shared machine; cap jobs, prefer CI, push early.
- SHOULD NOT leave throwaway `target/` directories behind; delete them after use.
