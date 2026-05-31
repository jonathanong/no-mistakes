# `no-mistakes/playwright-require-interactive-test-id`

Requires test IDs on interactive JSX elements.

Why: buttons, links, form controls, and similar elements need stable selectors
for reliable browser tests.

Counterexample: `<button onClick={save}>Save</button>`.

Fix: add the configured test ID attribute to the interactive element.
