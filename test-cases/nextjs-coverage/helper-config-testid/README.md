# helper-config-testid

Regression fixture for [#343](https://github.com/jonathanong/no-mistakes/issues/343).

The Playwright config is built by a helper (`createPlaywrightConfig`) that sets
`use.testIdAttribute = 'data-pw'` **inside the helper body**, so the attribute is
not statically readable from `playwright.config.ts`. The app marks elements with
`data-pw` and the spec locates them with `page.getByTestId(...)`.

Because the Playwright config's `testIdAttribute` cannot be read statically,
`getByTestId('save')` must fall back to the configured
`tests.playwright.selectors.testIds` (`data-pw`) so the `data-pw="save"` selector
is recognized as covered. Without the fallback, every `getByTestId`-based
assertion is reported as uncovered.
