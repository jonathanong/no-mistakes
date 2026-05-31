# `no-mistakes/playwright-literals`

Requires literal Playwright selector values.

Why: literal selectors allow coverage, related-test, and uniqueness analysis to
map tests to UI targets.

Counterexample: `page.getByTestId(buttonId)` or `<button data-pw={id} />`.

Fix: use string literals or supported static templates for test IDs and related
selector arguments.
