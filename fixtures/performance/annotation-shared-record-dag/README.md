# Shared-record annotation walk reproduction

These saved inputs reduce the expensive legacy `values::prefixes` walk sampled while
the full consumer initializes its MCP registry. They do not establish that this
is the only contributor to the full-consumer slowdown.

`shared-object-dag-30` contains one trusted SQL-builder leaf and 31 small shared
record nodes. An unknown conditional starts alternative-state comparison, whose
existing prefix walk revisits the shared subtrees. `dag-arguments-leaf` also
reproduces the walk with an arguments-object leaf and no SQL builder in the DAG.
The later annotated query only keeps annotation demand enabled.

On source `350ada01a`, both large cases reached an external 10-second cutoff.
Debugger samples showed 31 recursive `values::prefixes` frames, followed by
`values::changes` and `alternatives`. These are diagnostic reproductions under
concurrent build load, not matched performance measurements or infinite-loop
proofs. The large cases are manual stress inputs, not ordinary unit tests.

Run from the repository root, keeping an execution deadline:

```sh
no-mistakes --timeout 10 check --root fixtures/performance/annotation-shared-record-dag/shared-object-dag-30 --format json --jobs 1 --progress --verbose-timings
```

Negative controls preserve the depth but remove one property:

- `dag-no-join` omits the final conditional.
- `dag-primitive-leaf` uses a plain string instead of a reference-bearing builder.
- `dag-single-child` removes the duplicate child reference at each record level.

All three controls complete. `shared-object-dag-12` is the small, completing
fixture used to verify report parity and diagnostic work accounting in tests.
The unresolved imports are intentional configured executor/tag identities;
these fixtures need no installed dependencies or consumer checkout.

The observations do not justify a blanket visited-container shortcut. Prefix
map overwrites have ordering semantics, and before/after walks may use different
mutable argument arenas. The archived diagnostic baseline preserves the original traversal. The optimized
walk now has saved semantic regression guards under `prefix-semantics/`.

The `excluded-test-root` case separates evaluator demand from finding scope.
Its configuration excludes `**/*.test.mts`, yet the excluded test becomes an
evaluator root and its shared DAG reaches the same cutoff. Removing the test
completes. Replacing its DAG with an unannotated query also completes without
findings, while removing the exclusion produces exactly one missing-annotation
finding. Those controls show that the exclusion is valid but does not currently
limit evaluator roots. An eventual scope fix must retain excluded modules in
the import catalog: selected roots can still depend on them.

The optimized walk memoizes completed shared immutable records within each prefix
collection. A revisit replays only the record's direct prefix writes, preserving
last-writer order; argument-derived writes retain their original first-visit
behavior. Unfinished records reached through argument cycles are not cached.
Before and after collections use separate caches and argument arenas. Uniquely
owned records forward writes without retaining overlapping ancestor summaries.

The saved 30-level prefix and arguments DAGs have deterministic work-count tests
bounded by twice the 31 records plus two leaf visits. Small cases compare exact
prefix maps with the frozen original walk, including overwrite order, argument
cycles, sparse/deleted slots, both arenas, and a 128-record distinct-prefix chain.
These work bounds establish the reduced-case improvement; full-consumer resource
measurements and findings parity remain the acceptance gate.
