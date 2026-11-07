<!-- cspell:ignore rustup rustc heaptrack unprofiled -->
# Heap profiling

Start with `--verbose-timings` to locate expensive phases and files. Phase RSS
is a process-lifetime high-water mark; allocation stacks explain which code
allocates, how much remains live, and which temporary allocations create churn.
Use a fresh process per sample and the same repository revision, configuration,
command arguments, compiler, and worker count for comparisons.

## Build the release CLI

From the repository root, build only the CLI with default features and the
production release profile. Keep the existing allocator and Cargo profiles.
Enable debug information for stack attribution without disabling optimization:

```sh
rustup update stable
rustc --version
NO_MISTAKES_BUILD_NAPI=1 CARGO_PROFILE_RELEASE_DEBUG=1 \
  cargo build --release --locked -p no-mistakes --bin no-mistakes
```

The executable is `target/release/no-mistakes` unless `CARGO_TARGET_DIR` is set.
Use the matching executable and debug information when opening a recording.
Profile the built executable directly, so the recording excludes Cargo and
compilation. Record build settings with each profile: debug information changes
the build identity even though release optimization remains enabled. See the
[hosted measurement protocol](hosted-check.md) for controlled whole-command
comparisons.

## Linux: heaptrack

Install [KDE heaptrack](https://github.com/KDE/heaptrack) using your system's
package manager. From the analyzer checkout, replace `/path/to/project` with the
repository being analyzed and retain its intended check profile and arguments:

```sh
heaptrack target/release/no-mistakes check \
  --root /path/to/project --jobs 1 --verbose-timings --format json \
  > /tmp/no-mistakes-check.json 2> /tmp/no-mistakes-profile.stderr
```

Heaptrack prints the recording location and analysis command. Follow that
command, including any preprocessing requested by the installed version, then
open the recording with `heaptrack_gui` or inspect it with `heaptrack_print`.
Review allocation stacks, peak consumers and temporary allocations. Repeat with
the production worker count when investigating parallel memory pressure.
Heaptrack's interception adds overhead: compare instrumented runs with each
other, and confirm any performance improvement with unprofiled release runs.

## macOS: Instruments Allocations

Install Xcode and open Instruments through **Xcode → Open Developer Tool →
Instruments**. Choose **Allocations**, select the release `no-mistakes`
executable as the launch target, and configure its arguments, working directory
and environment to match the CLI run. For example, use `check --root
/path/to/project --jobs 1 --verbose-timings --format json`. Launch the process
from Instruments so startup allocations are included, record through command
completion, and save the trace.

Use the allocation timeline and call trees to inspect transient peaks and
persistent growth. Ensure Instruments can find the matching debug information
if stacks are unresolved. Apple's
[Analyze heap memory](https://developer.apple.com/videos/play/wwdc2024/10173/)
explains the Allocations workflow and measurement overhead. Compare equivalent
recordings and confirm changes outside Instruments.

## Share useful evidence

Report the source revision, Rust version, OS, build settings, command, worker
count and input size alongside the relevant phase and allocation stacks.
Distinguish peak RSS, live heap bytes, cumulative allocated bytes and allocation
counts; they measure different things. Inspect recordings and diagnostic paths
before sharing them: traces can contain source paths and input data. Publish
only sanitized summaries or approved recordings, using public fixture paths in
examples.

In a throwaway worktree, remove that worktree's `target/` after profiling and any
baseline comparison finishes. Do not use `cargo clean`, which may follow a shared
`CARGO_TARGET_DIR`, or remove the main workspace's build output.
