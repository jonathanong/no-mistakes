# ESLint Rules

`eslint-plugin-no-mistakes` keeps file-local TS/JS, React, Next.js, and
Playwright code static enough for AST analysis.

Use the plugin for file-local rules. Use [`no-mistakes check`](../cli/check.md)
when a rule needs repository graph or configuration context.

## Presets

The plugin exports two ESLint flat-config presets:

| Preset | Use when |
| --- | --- |
| `configs.recommended` | You want the default static-safety rules with low configuration. |
| `configs.strict` | You want every recommended rule plus stricter Playwright, React, test, and Next.js checks. |

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [noMistakes.configs.strict];
```

See [ESLint and Oxlint plugin](../eslint-plugin.md) for option schemas.

## Rule Index

| Rule | Purpose |
| --- | --- |
| [`await-array-methods`](await-array-methods.md) | Disallow awaiting synchronous array methods. |
| [`module-mock-boundary`](module-mock-boundary.md) | Enforce configured module mock boundaries. |
| [`module-mock-preserve-exports`](module-mock-preserve-exports.md) | Require module mock factories to preserve real exports. |
| [`nextjs-metadata-exports-location`](nextjs-metadata-exports-location.md) | Restrict Next.js metadata exports to route segment files. |
| [`nextjs-no-manual-script-tags`](nextjs-no-manual-script-tags.md) | Prefer `next/script` over raw JSX script tags. |
| [`nextjs-static-fetch-method`](nextjs-static-fetch-method.md) | Require static `fetch()` method options. |
| [`nextjs-static-fetch-url`](nextjs-static-fetch-url.md) | Require static `fetch()` URL arguments. |
| [`no-delete-property`](no-delete-property.md) | Disallow deleting object properties. |
| [`no-import-only-test-files`](no-import-only-test-files.md) | Disallow aggregate test files that only import tests. |
| [`no-placeholder-never-type-exports`](no-placeholder-never-type-exports.md) | Disallow exported `never` placeholder type aliases. |
| [`no-vitest-sequential`](no-vitest-sequential.md) | Disallow Vitest sequential modifiers. |
| [`playwright-assertion-timeout-cap`](playwright-assertion-timeout-cap.md) | Cap Playwright assertion timeouts. |
| [`playwright-consistent-attribute`](playwright-consistent-attribute.md) | Require a canonical test ID attribute. |
| [`playwright-defaults`](playwright-defaults.md) | Require literal defaults for prop-passed test IDs. |
| [`playwright-literals`](playwright-literals.md) | Require literal Playwright selector values. |
| [`playwright-naming-convention`](playwright-naming-convention.md) | Require a naming convention for literal test IDs. |
| [`playwright-no-empty`](playwright-no-empty.md) | Disallow empty test IDs. |
| [`playwright-no-set-timeout`](playwright-no-set-timeout.md) | Disallow fixed sleeps in Playwright tests. |
| [`playwright-prefer-get-by-test-id`](playwright-prefer-get-by-test-id.md) | Prefer `getByTestId` over CSS test-id selectors. |
| [`playwright-require-exported-component-attribute`](playwright-require-exported-component-attribute.md) | Require configured attributes in exported component JSX. |
| [`playwright-require-interactive-test-id`](playwright-require-interactive-test-id.md) | Require test IDs on interactive JSX elements. |
| [`playwright-selector-priority`](playwright-selector-priority.md) | Prefer semantic Playwright locators over raw selectors. |
| [`playwright-unique`](playwright-unique.md) | Require unique literal test IDs within a file. |
| [`react-no-iife-in-jsx`](react-no-iife-in-jsx.md) | Disallow immediately invoked functions inside JSX. |
| [`react-no-nullish-react-node`](react-no-nullish-react-node.md) | Disallow nullish coalescing on ReactNode-like values. |
| [`react-no-use-promise-resolve`](react-no-use-promise-resolve.md) | Disallow `React.use(Promise.resolve(...))`. |
| [`test-no-error-message-matching`](test-no-error-message-matching.md) | Disallow assertions on error message strings. |
| [`test-no-shared-state`](test-no-shared-state.md) | Disallow mutable module-scope test state. |
| [`ts-no-export-renaming`](ts-no-export-renaming.md) | Disallow value export renaming. |
| [`ts-no-function-aliases`](ts-no-function-aliases.md) | Disallow function wrappers that only alias another function. |
| [`vitest-mock-test-file-naming`](vitest-mock-test-file-naming.md) | Require `.mock.test` filenames for module-mocking tests. |
