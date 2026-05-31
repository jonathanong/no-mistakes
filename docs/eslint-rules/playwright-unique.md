# `no-mistakes/playwright-unique`

Requires unique literal test IDs within a file.

Why: duplicate IDs make selector coverage ambiguous.

Counterexample: two elements in one file with `data-testid="save"`.

Fix: rename duplicate literal values or split repeated UI into scoped selectors.
