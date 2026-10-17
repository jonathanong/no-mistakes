# PostgreSQL evaluator scope frontiers

Completed inert module templates retain a proof in the existing builder-index
cache. The proof admits only scalar values and functions with symbolic module
captures after both root evaluation and global reconstruction. An ordinary empty
builder set is not equivalent: lexical closures and argument objects may contain
no builder IDs while retaining other live state.

A reachability boundary that explicitly seeds every module (`AllModules`) can
skip the proven scope's binding values. Captured-binding origins and mapped
argument objects remain dynamic and are always traversed. Selected-root walks,
including alternative module-initial snapshots, still follow each actual owner.
Every mutable scope view clears the proof through the existing copy-on-write
invalidation path; complete scope snapshots carry proof and values together.

The `postgres.annotation.reachability_binding_visits` verbose work counter counts
bindings in the scan branch, aggregated once per collection. It excludes skipped
binding loops and does not count every other evaluator traversal. Template
construction remains a cold validation cost. Existing cache layout is reused
without adding a field to each scope allocation.

Saved small/large fixtures isolate repeated immutable-module scans, and rule
fixtures cover nested executor alternatives and pristine independent entrypoints. Work counts
prove an algorithmic reduction, not a runtime improvement. Matched production
cold/control/consuming wall, CPU, and peak-RSS measurements are required before
accepting a performance claim. This is a restricted immutable-frontier phase;
it does not implement undo checkpoints or a complete generational collector.
