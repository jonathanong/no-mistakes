# `no-mistakes/playwright-require-exported-component-attribute`

Requires exported components to render at least one configured selector
attribute.

Why: exported UI components without tracked attributes are difficult to connect
to Playwright coverage.

Counterexample: `export function Button() { return <button>Save</button>; }`.

Fix: add a configured attribute to the returned JSX tree, or disable the rule for
intentional non-interactive components.
