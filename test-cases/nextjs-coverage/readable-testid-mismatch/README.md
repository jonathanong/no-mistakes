# readable-testid-mismatch

Companion to `helper-config-testid` for [#343](https://github.com/jonathanong/no-mistakes/issues/343).

Here the Playwright config sets `use.testIdAttribute = 'data-qa'` directly in the
exported object, so it **is** statically readable. The app marks elements with
`data-pw` (the configured `selectors.testIds`).

Because the readable `testIdAttribute` (`data-qa`) takes precedence over the
`selectors.testIds` fallback, `getByTestId('save')` resolves to `data-qa` and does
**not** cover the `data-pw="save"` selector — verifying the fallback does not
over-attribute coverage. An explicit `tests.playwright.testIdAttribute: data-pw`
overrides even the readable value and restores coverage.
