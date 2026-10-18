# Annotation counter parity

These complete `postgres.annotation.*` maps were captured from the production
CLI at `ef62b0c5a3a86ad684477a61e80f28df7687556e`, using the unchanged
`postgres-annotations` and `postgres-branch-copies` fixtures with
`check --verbose-timings --format json`. `postgres.annotation.parallel_passes`
is the later fixed-slot observation from the parity test: both fixtures record
1. Absent metrics intentionally stay absent; moving counters to fixed storage
must not expose unused zero entries.

The fixture test also compares observed and unobserved check reports. Separate
counter tests exercise all eight registered metrics, including explicit zeros
and parallel increments, and assert that extension storage stays uninitialized.

The baseline includes the root-pruning optimization from #2134. The frozen
production binary was built at `f641adde73f3e293b2ccdf178ca034b4336865de`,
whose complete source tree is identical to that baseline merge commit.
