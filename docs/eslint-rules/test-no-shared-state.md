# `no-mistakes/test-no-shared-state`

Disallows mutable module-scope test state.

Why: shared mutable state makes tests order-dependent and blocks reliable
parallel execution.

Counterexample: `let user; beforeEach(() => { user = ... })` at module scope.

Fix: create state inside each test or reset it through explicit setup/cleanup.
