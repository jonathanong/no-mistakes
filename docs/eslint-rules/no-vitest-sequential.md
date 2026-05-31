# `no-mistakes/no-vitest-sequential`

Disallows Vitest sequential modifiers.

Why: sequential tests usually signal hidden shared state and reduce parallel,
deterministic test execution.

Counterexample: `describe.sequential("users", () => {})`.

Fix: isolate test state and remove `.sequential`.
