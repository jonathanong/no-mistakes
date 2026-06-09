# override-testid-attribute

Companion to `readable-testid-mismatch` for [#343](https://github.com/jonathanong/no-mistakes/issues/343).

Same setup as `readable-testid-mismatch` (the Playwright config statically sets
`testIdAttribute: 'data-qa'`), but the no-mistakes config declares an explicit
`tests.playwright.testIdAttribute: data-pw`. The explicit override has the highest
precedence, so `getByTestId('save')` resolves to `data-pw` and covers the
`data-pw="save"` selector despite the readable `data-qa`.
