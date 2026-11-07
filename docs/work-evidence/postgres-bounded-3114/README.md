# Bounded key proofs: issue 3114

<!-- cspell:ignore rustc -->

Baseline: fresh `origin/main`,
`759777752f2399d9b83f17e27b0ac8af6a9fe6d7`, Rust
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`, default Cargo features and dev profile.
The baseline CLI SHA256 is
`50ed9806684201fb8a0524972e8b2f46a310e1b59fb7e99d2367eb1481d80e54`.

All requested shapes already pass on this baseline. No analyzer, binding,
declaration, manifest, or dependency changes are needed. The change retains
saved regressions and documents the existing proofs.

| Saved SQL | Accepted statement lines | Still-reported control lines | Findings |
| --- | --- | --- | --- |
| drivers.sql | 1, 2, 3 | 4 | 2 |
| cte-writes.sql | 1, 2, 3 | 4, 5 | 2 |
| unnest.sql | 1, 2 | 3 | 1 |
| unqualified.sql | 1 | 2 | 2 |
| indexes.sql | 1, 2, 3, 4 | 5, 6, 7 | 3 |

The five files contain 13 accepted statements and eight negative controls.
Each configuration exits 1 because its controls still report.
`off.yml` exits 0 with empty findings. Existing `valid.yml`,
`invalid.yml`, and `keys.yml` controls produce 0, 3, and 5 findings.

Earlier fixes checked: #2671 (closes #2644), #2698 (closes #2664),
#2705 (closes #2666), and #2716 (closes #2704).

## Reproduction and instruction counts

Build both revisions with the same toolchain, features, profile, and job count.
Heavy Cargo commands use `flock /tmp/nm-heavy.lock`,
`CARGO_BUILD_JOBS=6`, and task-owned `CARGO_TARGET_DIR=/tmp/nm-3114-target`.
The target directory is removed after validation.

Use a writable runtime directory for the CLI invocation lock in a sandbox.
For each saved shape, run the following against the baseline and candidate
binaries, retaining default effort and worker count (no effort or jobs flags):

```sh
mkdir -p /tmp/nm-3114-runtime
export XDG_RUNTIME_DIR=/tmp/nm-3114-runtime
root="$PWD/test-cases/rules/postgres-bounded-statements/issue-3114"
bin=/tmp/nm-3114-main
for shape in drivers cte-writes unnest unqualified indexes off; do
  perf stat -r 5 -e instructions -- "$bin" check --root "$root" \
    --config "$root/$shape.yml" --format json > /dev/null
done
```

The expected control exit 1 does not invalidate hardware counts when counters
are available. Run in balanced baseline/candidate/candidate/baseline order.

Before and after instruction counts are **unavailable** on this host.
`perf stat` fails with “No supported events found” because
`perf_event_paranoid=4`, including outside the sandbox; `sudo -n perf`
requires credentials. All 12 before/after commands for the five shapes and rule-off were attempted.
No kernel settings were changed, and no unchanged-count or speedup claim
is made. Timing observations are not a substitute for instruction counts.

Production sources are unchanged, so no performance optimization is claimed.
The comparison artifact records output/status equality separately from this
unavailable hardware measurement.

## Validation environment

An existing `/tmp/.git` directory changes the ancestor-ignore fixture's setup.
Its existing regression fails with the default temp root and passes with
`TMPDIR=/var/tmp/nm-3114-tests`, which has no Git ancestor. Full validation
uses that isolated test directory while Cargo artifacts stay under `/tmp`.
The installed ShellCheck 0.11.0 binary is selected explicitly because its
default mise shim has no selected version. AST checks use CI's 0.44.1.

The matching default-profile rebuild is byte-identical to the baseline CLI.
All nine saved stdout/stderr/status comparisons remain identical, including
rule-off. Hardware counts remain unavailable despite all 12 attempts.

Full locked all-features validation passes: 13,262 library tests (three existing
ignored), 488 binary tests, and 1,965 integration tests, including snapshot
reference validation. All 17 focused rule-doc snippet tests pass. After
strengthening the expression-index control to `LENGTH(slug) = $1`, the five
snapshots and all 629 PostgreSQL CLI tests pass again with updates disabled.
Repository check output is empty; spelling and AST guards pass.

Use `flock --close /tmp/nm-heavy.lock` for subsequent heavy commands: a completed
validation's idle compiler-cache daemon retained an inherited lock descriptor.
The idle holder was stopped after verifying it had no compiler children;
closing the descriptor in command children prevents recurrence.
