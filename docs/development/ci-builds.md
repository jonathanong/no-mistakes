# Debug metadata in CI builds

Workspace development and test profiles set `debug = 0`, so local builds, CI,
and release validation omit debugger metadata. This reduces compilation work and
temporary artifact size, at the cost of file/line and inline-frame detail in
native backtraces. Debug assertions, overflow checks, features, and optimization
levels retain their existing settings. Distributed release profiles are unchanged.

For local debugging, set `CARGO_PROFILE_DEV_DEBUG=2` or
`CARGO_PROFILE_TEST_DEBUG=2` when invoking Cargo to restore debugger symbols.

CI and release validation share `.github/scripts/rust-validation.sh`. Coverage
instrumentation is applied only to the coverage build and test steps; documentation tests
run separately in both workflows. Resumed releases may use historical source
tags without the shared helpers or profile defaults. Release validation checks
out the running workflow's revision (`github.workflow_sha`) separately into
`validation-tools`, then moves that checkout to the runner's temporary directory
before validation. Repository-wide checks therefore inspect only the tagged
source. Cargo runs against the extracted release source in the workspace root.
Release job environment overrides enforce the same symbol-free
profiles for those historical manifests. The tooling checkout never replaces the tagged Rust source or becomes Cargo's
working directory.

The real-addon API steps load `inline-owned-member-api.test.js` through
`real-napi-api.test.js` on Linux, macOS, and Windows with
`NO_MISTAKES_TEST_NAPI_ADDON_PATH` pointing to the compiled addon. This exercises async CJS/ESM method-acquisition parity; ordinary
JavaScript discovery alone skips the native behavior test without that addon.
Those steps also run `sql-verdicts-api.test.js` to compare async CJS/ESM check
reports with the sibling compiled CLI, including SQL advisory aggregation.
The Linux JavaScript job builds the CLI before the addon so both artifacts are
available to these parity tests, as in the native platform matrix.

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
not represent a clean standalone JavaScript job. These results use the previous pinned compiler. The same comparison was
repeated on Rust 1.99 after the latest-stable policy changed, as recorded below.


## Rust 1.99 confirmation

[Run 38333716822](https://github.com/jonathanong/no-mistakes/actions/runs/38333716822)
used the same production source and reversed-order design, changing only the
measurement workflow to install Rust 1.99.0 (LLVM 23.1.1). Its measurement commit
was `09492537c61143aa3e893d8c1d8fc4d9e8bfb486`.

| Workload | Runner A: debug 2 / 0 | Runner B: debug 2 / 0 |
| --- | --- | --- |
| Clean CLI wall time | 153.74 / 115.20 s | 174.58 / 140.54 s |
| CLI user + system CPU | 373.56 / 272.60 s | 455.58 / 355.56 s |
| CLI peak RSS | 8,142,892 / 5,214,376 KiB | 7,720,372 / 5,447,676 KiB |
| Addon wall time after CLI build | 90.53 / 58.43 s | 97.93 / 65.33 s |
| Addon user + system CPU | 162.49 / 98.18 s | 189.58 / 121.46 s |
| Addon peak RSS | 7,839,504 / 5,540,288 KiB | 7,992,332 / 5,614,572 KiB |

CLI size fell from 1,219,707,480 to 308,865,720 bytes; addon size from
1,163,613,904 to 288,184,704 bytes. Complete target-directory size fell from
6,968,360,260 to 2,465,067,906 bytes, identically on both runners. Version
outputs and async addon responses matched. Clean CLI builds were 34–39 seconds
faster; the dependency-warm addon rebuild was 32–33 seconds faster. These are
again two paired observations with the scope and limitations stated above.
