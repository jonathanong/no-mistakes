---
name: rust-rules
description: "MUST NOT / SHOULD NOT rules for Rust: lints, parallelism, hot loops, determinism, coverage, and performance claims. Use when writing or reviewing Rust code."
---

# Rust rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- SHOULD NOT silence lints with inline `#[allow]`; fix the code. Warnings fail the build.
- MUST NOT lock a global `Mutex<HashMap>` or other process-wide lock per item inside parallel iterators; shard or use concurrent maps.
- MUST NOT read files, spawn processes, compile regexes, or reparse inside a hot loop; hoist them (`LazyLock`/`OnceLock` for regexes).
- MUST NOT rebuild a repo-wide index or do linear scans per item; index once, then look up.
- MUST NOT emit nondeterministic output from parallel work; sort or merge before rendering.
- MUST NOT block an async runtime or the Node event loop with CPU-heavy native calls; run them on a thread pool.
- MUST NOT exclude files from coverage (`--ignore-filename-regex`); refactor them to meet the gate.
- MUST NOT claim a speedup without matched before/after measurements with instrumentation off.
- SHOULD NOT write fixture files from test code; save fixtures and copy them in.
