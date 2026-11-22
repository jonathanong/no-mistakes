# PostgreSQL annotation call values

The saved inputs isolate two ways max-effort evaluation can reach a scalar
record shared by both properties of its parent. The source grows linearly with
depth, while walking every reference to each child grows exponentially.

- `scalar-refresh` passes a depth-20 record as the second executor operand.
  This exercises the builder refresh of earlier operands after argument
  evaluation. The first operand is an unannotated literal query and produces
  one finding. [Fixture source](scalar-refresh/root.mts)
- `scalar-opaque` applies unary `+` to a depth-28 record in its own statement.
  Opaque coercion examines the operand for callbacks. A separate unannotated
  literal query produces one finding. The `void` statement keeps `+` from
  continuing the preceding initializer. [Fixture source](scalar-opaque/root.mts)

Refresh visits zero values when both mutation tables are empty and one when
an unrelated builder changes and the input is a scalar record. Opaque callback
traversal visits only the root when the record contains neither a callback
environment nor an argument identity. These visit counts are the regression
ceilings. [Traversal tests](../../../crates/no-mistakes/src/codebase/postgres/query_annotation/evaluate/effects/call_value_work_tests.rs)

The Criterion `annotation_call_values` group checks each finding set and
measures fresh max-effort requests for these inputs and the multi-root workload.
[Shared callback payload](../annotation-fact-scale/README.md)

For a release comparison, set `binary` to a frozen executable and run from the
repository root. Use each fixture, `low` and `max`, and `--jobs 1` and `--jobs 8`.

```sh
binary=/path/to/no-mistakes
fixture="$PWD/fixtures/performance/annotation-scalar-call-values/scalar-refresh"
effort=max
jobs=1
/usr/bin/time -f 'wall=%e user=%U system=%S peak_rss_kib=%M' \
  "$binary" check --root "$fixture" --config "$fixture/$effort.yml" \
  --jobs "$jobs" --timeout 0 --lock-timeout 0 --format json > report.json
```

Use baseline, candidate, candidate, baseline order with fresh invocations and
observer diagnostics disabled. Check that exit status is 1 and report bytes
match across efforts and binaries. Compare user plus system CPU for the
max-to-low ratio and keep wall time and peak RSS with each sample.
