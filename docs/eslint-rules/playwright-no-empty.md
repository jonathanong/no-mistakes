# `no-mistakes/playwright-no-empty`

Disallows empty test IDs.

Why: empty selector values cannot identify stable UI targets.

Counterexample: `<button data-testid="" />`.

Fix: provide a meaningful literal test ID or remove the unused attribute.
