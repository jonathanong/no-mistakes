# Equal records with distinct identities

`root.mts` builds two separately allocated but structurally equal record DAGs.
Each level repeats its own preceding node. The conditional expression forces a
join between the two roots, so pointer identity alone cannot avoid repeating
structural comparisons. The probe reads through all 12 joined record levels and
must retain the exact annotated SQL leaf alongside the independent static query.
Returning an unknown or dropping the joined binding cannot satisfy that assertion.

The AST regression declares the production supplied-source whole-SQL profile:
`executed_source/imports.rs` sets `annotation.precise_records = true`. That
profile resolves static record members and deduplicates structurally equal
alternative results. The existing annotation-only regression keeps its original
`precise_records = false` setting. Ordinary annotation checks conservatively
retain aggregate member candidates; the CLI config here retains that historical
`unanalyzableSql: ignore` policy. Its clean report is a parity control, not proof
that the ordinary profile resolves the deep leaf.

`state.json` saves the same 12-level record topology with an explicit SQL leaf
for the equality work-count regression. The test constructs immutable evaluator
values from those saved nodes, rather than flattening the DAG into a tree.
The frozen comparator performs 8,191 value comparisons for 12 distinct container
pairs. The optimized comparison keeps completed pairs within that one equality
call; shallow comparisons use an inline memo, and larger comparisons spill to a
local hash set. Record labels and wrapper variants still compare separately.

Run the focused tests with:

```sh
cargo test --locked -p no-mistakes --lib value::equality:: -- --nocapture
```

The tests compare against the frozen structural oracle, exercise a changed leaf,
changed record labels, wrapper variants, container length changes and independent
copy-on-write snapshots, and check the actual saved source's query events and
comparison counts. Mutable argument arrays are not followed by equality: their
IDs retain the existing semantics.

This fixture isolates repeated equality work. A work-count reduction and local
microbenchmark do not establish a completed hosted check or a consumer speedup.
Production measurements must use matched binaries, compiler flags, source trees,
configuration, thread counts and diagnostics settings.
