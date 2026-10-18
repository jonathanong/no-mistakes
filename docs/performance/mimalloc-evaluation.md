<!-- cspell:ignore Mimalloc mimalloc codegen Vouchington -->

# Mimalloc allocator evaluation

The CLI and N-API library use mimalloc through Rust's global allocator when the
default `mimalloc` feature is enabled. The `override` feature is not enabled, so
the C allocator used by Node remains unchanged. Rust-library consumers that
provide their own global allocator can opt out with
`--no-default-features --features mermaid-validation`.

The comparison uses the same source revision and lockfile for both variants.
The system-allocator control disables default features and explicitly retains
`mermaid-validation`; the candidate uses the default features. Only the allocator
feature differs. Linux arm64 uses release fat LTO and one codegen unit. macOS
arm64 uses thin LTO and 16 codegen units, matching release builds.

The native evaluation workflow builds both the CLI and N-API addon, checks an
allocator probe with the feature off, without global-allocator registration,
and with registration, then runs N-API parity tests for both variants. It measures
the pinned Vouchington consumer and both public hosted-check fixtures with
`--jobs 1`, `4`, and `auto`. Each invocation saves its executable hash,
platform and CPU facts, exit status, normalized report and warnings, wall time,
user/system CPU time, and per-process peak RSS. The ARM table is published here
after the matched native runs finish; no timing or memory improvement is
claimed before those results are reviewed.

The saved probe calls `mi_is_in_heap_region` only while its `Vec` remains live.
It proves the crate's allocator registration is linked into the measured CLI
and addon build. A dependency-only control without the registration must fail.
