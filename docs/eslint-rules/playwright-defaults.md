# `no-mistakes/playwright-defaults`

Requires literal defaults for prop-passed test IDs.

Why: component selector analysis can track literal defaults through component
props but cannot safely infer arbitrary dynamic values.

Counterexample: `function Button({ testId = makeId() }) { ... }`.

Fix: provide literal defaults for test ID props or pass literal selector values
at call sites.
