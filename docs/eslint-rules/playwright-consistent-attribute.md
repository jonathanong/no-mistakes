# `no-mistakes/playwright-consistent-attribute`

Requires a canonical test ID attribute.

Why: a single selector attribute keeps test coverage and selector extraction
predictable.

Counterexample: using both `data-testid` and `data-pw` when `data-pw` is canonical.

Fix: replace alternate test ID attributes with the configured canonical
attribute, such as `data-pw`.
