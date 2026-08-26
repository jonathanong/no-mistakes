# ESLint And Oxlint Rules

`eslint-plugin-no-mistakes` keeps local source patterns deterministic and easy
to review. Use `configs.recommended` for the baseline rules and
`configs.strict` for the broader set; both presets and installation are
documented in [`eslint-plugin`](../eslint-plugin.md).

Every rule page states the reason for the rule, exact disallowed and allowed
forms, options, fix guidance, suppression, and related rules. Rules with
options are configured through normal ESLint rule options; the exhaustive
cross-rule option reference is in [`eslint-plugin`](../eslint-plugin.md#rule-options).

## Async and module boundaries

| Rule | Purpose |
| --- | --- |
| [`async-call-disposition`](async-call-disposition.md) | Make configured async calls awaited, returned, or explicitly detached. |
| [`async-try-catch-return-await`](async-try-catch-return-await.md) | Keep configured promise rejections inside the intended `try`/`catch`. |
| [`await-array-methods`](await-array-methods.md) | Reject awaiting synchronous array helpers. |
| [`module-mock-boundary`](module-mock-boundary.md) | Restrict configured internal module mocks. |
| [`module-mock-preserve-exports`](module-mock-preserve-exports.md) | Preserve untouched exports in allowed internal module mocks. |

## Next.js and server boundaries

| Rule | Purpose |
| --- | --- |
| [`nextjs-metadata-exports-location`](nextjs-metadata-exports-location.md) | Keep Next.js metadata exports in route segment modules. |
| [`nextjs-no-manual-script-tags`](nextjs-no-manual-script-tags.md) | Prefer `next/script` to raw JSX script tags. |
| [`nextjs-static-fetch-method`](nextjs-static-fetch-method.md) | Require statically visible fetch methods. |
| [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) | Require statically visible fetch URLs. |
| [`no-banned-import-outside-allowed-paths`](no-banned-import-outside-allowed-paths.md) | Restrict configured imports to approved helper paths. |
| [`no-global-fetch-outside-helper`](no-global-fetch-outside-helper.md) | Restrict global fetch to approved helper paths. |
| [`server-require-nullable-fetch-wrapper`](server-require-nullable-fetch-wrapper.md) | Require configured nullable getters to use a common wrapper. |

## Playwright and UI selectors

| Rule | Purpose |
| --- | --- |
| [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md) | Cap assertion-specific waits. |
| [`playwright-consistent-attribute`](playwright-consistent-attribute.md) | Require one canonical test-id attribute. |
| [`playwright-defaults`](playwright-defaults.md) | Require literal defaults for passed-through test IDs. |
| [`playwright-literals`](playwright-literals.md) | Require statically analyzable test-id values. |
| [`playwright-naming-convention`](playwright-naming-convention.md) | Enforce test-id naming. |
| [`playwright-no-empty`](playwright-no-empty.md) | Reject empty test IDs. |
| [`playwright-no-set-timeout`](playwright-no-set-timeout.md) | Reject fixed Playwright sleeps. |
| [`playwright-prefer-get-by-test-id`](playwright-prefer-get-by-test-id.md) | Prefer `getByTestId` to exact test-id CSS selectors. |
| [`playwright-require-exported-component-attribute`](playwright-require-exported-component-attribute.md) | Require configured hooks in exported component JSX. |
| [`playwright-require-interactive-test-id`](playwright-require-interactive-test-id.md) | Require hooks on interactive JSX elements. |
| [`playwright-selector-priority`](playwright-selector-priority.md) | Prefer semantic Playwright locators. |
| [`playwright-unique`](playwright-unique.md) | Reject duplicate literal test IDs in a file. |

## PostgreSQL safety

| Rule | Purpose |
| --- | --- |
| [`postgres-cursor-call-contract`](postgres-cursor-call-contract.md) | Require direct annotated SQL for cursor executors. |
| [`postgres-no-manual-transaction`](postgres-no-manual-transaction.md) | Keep transaction lifecycle in a dedicated helper. |
| [`postgres-no-unbounded-query-fanout`](postgres-no-unbounded-query-fanout.md) | Bound concurrent mapped database queries. |

## React and tests

| Rule | Purpose |
| --- | --- |
| [`no-import-only-test-files`](no-import-only-test-files.md) | Reject aggregate test files that only import other tests. |
| [`no-vitest-sequential`](no-vitest-sequential.md) | Reject Vitest sequential modifiers. |
| [`react-no-iife-in-jsx`](react-no-iife-in-jsx.md) | Reject IIFEs in JSX. |
| [`react-no-nullish-react-node`](react-no-nullish-react-node.md) | Preserve nullish ReactNode semantics. |
| [`react-no-use-promise-resolve`](react-no-use-promise-resolve.md) | Reject `React.use(Promise.resolve(...))`. |
| [`test-no-error-message-matching`](test-no-error-message-matching.md) | Prefer stable error contracts over message text. |
| [`test-no-shared-state`](test-no-shared-state.md) | Reject mutable module-scope test state. |
| [`vitest-mock-test-file-naming`](vitest-mock-test-file-naming.md) | Name module-mocking tests `*.mock.test.*`. |

## TypeScript API shape

| Rule | Purpose |
| --- | --- |
| [`no-delete-property`](no-delete-property.md) | Avoid in-place object-shape deletion. |
| [`no-placeholder-never-type-exports`](no-placeholder-never-type-exports.md) | Reject exported placeholder `never` aliases. |
| [`ts-no-export-renaming`](ts-no-export-renaming.md) | Keep value-export identities direct. |
| [`ts-no-function-aliases`](ts-no-function-aliases.md) | Reject wrappers that only alias another call. |
| [`ts-preserve-null-option-defaults`](ts-preserve-null-option-defaults.md) | Preserve explicit null option values. |
