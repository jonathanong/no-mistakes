---
name: rust-rules
description: "Load before writing or reviewing Rust code, Cargo.toml, clippy settings, or benchmarks. Lists the defects to avoid and the patterns that need a stated reason: lints, parallel locking, hot loops, repo-wide indexes, deterministic output, blocking the async runtime or Node event loop, coverage exclusions, speedup claims, test fixtures."
---

# Rust rules

MUST NOT is a defect. SHOULD NOT needs a stated reason.

- SHOULD NOT suppress a lint without a stated reason. Prefer `#[expect]` when the supported Rust version provides it and the lint is expected to fire.
- SHOULD NOT take a process-wide lock per item in a parallel iterator when contention limits throughput.
- MUST NOT read files, spawn processes, compile regexes, or reparse inside a hot loop; hoist them (`LazyLock`/`OnceLock` for regexes).
- SHOULD NOT rebuild a shared index or linearly scan a growing collection for each item when an index built once can answer the lookups.
- SHOULD NOT emit nondeterministic ordering from parallel work when callers rely on stable output order.
- MUST NOT block an async runtime or the Node event loop with CPU-heavy native calls; run them on a thread pool.
- MUST NOT claim a speedup without matched before/after measurements with instrumentation off.
