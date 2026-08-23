# `no-mistakes/no-three-sequential-awaits`

Disallows three sequential `await` statements in the same block.

Why: a run of three awaits usually means the work should be parallelized with
`Promise.all` or extracted behind a helper that names the dependency chain.

Counterexample: `await one(); await two(); await three();`.

Fix: `await Promise.all([one(), two(), three()])`, or extract the dependent
steps into a function.
