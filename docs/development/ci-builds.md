# Debug metadata in CI builds

The JavaScript API and AST analysis jobs set `CARGO_PROFILE_DEV_DEBUG=0` before
Rust cache restore and compilation. These jobs execute CLI/addon behavior and
do not consume debugger metadata. This reduces compilation work and temporary
artifact size, at the cost of file/line and inline-frame detail in native CI
backtraces. Debug assertions, overflow checks, features, and optimization levels
retain their existing settings. Local development and distributed release
profiles are unaffected.

## Hosted measurement

[Run 38332918032](https://github.com/jonathanong/no-mistakes/actions/runs/38332918032)
built source `202d9ca7140af38c38bc8b49f5ec700998b6029b` plus only the experiment
workflow (measurement commit `1d68ff3807c2c65556a79877d6e867c3890fb819`) with
Rust 1.96.0 on two four-vCPU `ubuntu-latest` x86 runners. Each runner used a
fresh Cargo target directory per profile, default features, and `-D warnings`.
The orders were debug 2 then 0, and debug 0 then 2. Dependencies in the Cargo
registry could be warm for the second build; reversing order checks that bias.

| Workload | Runner A: debug 2 / 0 | Runner B: debug 2 / 0 |
| --- | --- | --- |
| Clean CLI wall time | 203.31 / 155.24 s | 188.08 / 147.51 s |
| CLI user + system CPU | 532.74 / 392.94 s | 494.74 / 369.29 s |
| CLI peak RSS | 8,280,396 / 5,297,704 KiB | 8,056,056 / 5,316,976 KiB |
| Addon wall time after CLI build | 108.84 / 73.13 s | 104.67 / 69.44 s |
| Addon user + system CPU | 223.39 / 138.56 s | 216.31 / 127.43 s |
| Addon peak RSS | 7,582,760 / 5,475,844 KiB | 7,514,576 / 5,562,584 KiB |

CLI size fell from 1,022,593,496 to 241,485,928 bytes; addon size from
972,375,904 to 222,004,936 bytes. Complete target-directory size fell from
6,037,581,967 to 2,046,593,000 bytes. Both runners produced identical size
results. CLI version output and the addon's async version response matched.
The ordinary CI tests provide broader behavior validation.

This is two paired observations, not a build-time guarantee. The addon step
rebuilds the library with N-API enabled after the CLI build, so its timing does
not represent a clean standalone JavaScript job. These results predate the
latest-stable compiler policy; a Rust 1.99 confirmation run is being collected.
