# ESLint and Oxlint Plugin

The lint plugin enforces file-local code shapes that keep the CLI analyzers
deterministic. Use it in editors and CI; use the CLIs for project-wide graph
checks.

## `eslint-plugin-no-mistakes`

Rules for Playwright test IDs, static Next.js fetch calls, TypeScript export
clarity, and ReactNode fallback semantics.

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

ESLint flat config:

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [
  {
    files: ["**/*.{js,jsx,ts,tsx,mjs,mts}"],
    plugins: { "no-mistakes": noMistakes },
    rules: noMistakes.configs.strict.rules,
  },
];
```

Oxlint:

```jsonc
{
  "jsPlugins": ["eslint-plugin-no-mistakes"],
  "rules": {
    "no-mistakes/playwright-literals": "error",
    "no-mistakes/nextjs-static-fetch-url": "error",
    "no-mistakes/ts-no-export-renaming": "error",
    "no-mistakes/react-no-nullish-react-node": "error",
  },
}
```

### Rules

| Rule                                                          | Purpose                                                                                 |
| ------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `no-mistakes/await-array-methods`                             | Disallows `await` on known synchronous array methods such as `sort()` and `slice()`.    |
| `no-mistakes/playwright-literals`                             | Requires JSX test IDs and `getByTestId()` arguments to be static.                       |
| `no-mistakes/playwright-defaults`                             | Requires prop-passed test IDs to have string-literal defaults.                          |
| `no-mistakes/playwright-unique`                               | Requires literal test IDs to be unique within a file.                                   |
| `no-mistakes/playwright-no-empty`                             | Disallows empty literal test IDs.                                                       |
| `no-mistakes/playwright-consistent-attribute`                 | Requires one canonical test ID attribute.                                               |
| `no-mistakes/playwright-require-exported-component-attribute` | Requires exported components to return JSX containing configured attributes.            |
| `no-mistakes/playwright-require-interactive-test-id`          | Requires test IDs on interactive JSX elements.                                          |
| `no-mistakes/playwright-prefer-get-by-test-id`                | Reports exact CSS test ID selectors passed to Playwright APIs.                          |
| `no-mistakes/playwright-naming-convention`                    | Requires literal test IDs to match a regex.                                             |
| `no-mistakes/playwright-assertion-timeout-cap`                | Caps per-assertion timeout options.                                                     |
| `no-mistakes/playwright-no-set-timeout`                       | Disallows fixed `setTimeout()` sleeps in Playwright tests.                              |
| `no-mistakes/playwright-selector-priority`                    | Prefers semantic Playwright locators over CSS/text selectors.                           |
| `no-mistakes/nextjs-metadata-exports-location`                | Restricts `metadata` exports to Next.js route segment files.                            |
| `no-mistakes/nextjs-no-manual-script-tags`                    | Prefers `next/script` over raw JSX `<script>` tags.                                     |
| `no-mistakes/nextjs-static-fetch-url`                         | Requires `fetch()` URL arguments to be string literals or expression-free templates.    |
| `no-mistakes/nextjs-static-fetch-method`                      | Requires `fetch()` `method` options to be string literals or expression-free templates. |
| `no-mistakes/no-delete-property`                              | Disallows `delete obj.prop` shape mutation.                                             |
| `no-mistakes/no-import-only-test-files`                       | Disallows test aggregate files that only import other tests.                            |
| `no-mistakes/no-placeholder-never-type-exports`               | Disallows exported `never` placeholder type aliases.                                    |
| `no-mistakes/no-vitest-sequential`                            | Disallows `.sequential` test modifiers.                                                 |
| `no-mistakes/react-no-iife-in-jsx`                            | Disallows IIFEs inside JSX expressions.                                                 |
| `no-mistakes/ts-no-export-renaming`                           | Disallows value export aliases such as `export { X as Y }`.                             |
| `no-mistakes/ts-no-function-aliases`                          | Disallows wrappers that only forward to another function.                               |
| `no-mistakes/react-no-nullish-react-node`                     | Disallows `??` on explicitly typed ReactNode values.                                    |
| `no-mistakes/react-no-use-promise-resolve`                    | Disallows `React.use(Promise.resolve(...))`.                                            |
| `no-mistakes/test-no-error-message-matching`                  | Disallows assertions against `err.message` strings.                                     |
| `no-mistakes/test-no-shared-state`                            | Disallows mutable module-scope test state.                                              |
| `no-mistakes/vitest-mock-test-file-naming`                    | Requires `.mock.test.*` filenames when tests use mocking APIs.                          |

`configs.recommended` enables the deterministic defaults for all domains.
`configs.strict` also enables canonical Playwright attribute, naming,
interactive element, and `getByTestId` preference rules.

Playwright selector rules default to `data-testid` and `data-pw`. Override
selectors per rule:

```js
{
  "no-mistakes/playwright-literals": ["error", {
    selectorAttributes: ["data-pw", "data-qa"],
    allowDefaultedProps: true,
    allowStaticTemplates: false
  }]
}
```

Require exported components to expose a stable hook in each JSX return branch:

```js
{
  "no-mistakes/playwright-require-exported-component-attribute": ["error", {
    attributes: ["data-pw"],
    components: ["/Button$/"],
    ignoreComponents: ["Layout"],
    wrappers: ["memo", "forwardRef", "observer"],
    allowSpreadAttributes: false,
    exportTypes: ["named", "default"],
    checkAnonymousDefault: false
  }]
}
```

The rule checks exported PascalCase components defined in the current file.
Branches that return `null` are ignored; every JSX-returning branch must include
one configured attribute somewhere in that returned JSX tree.

Use `no-mistakes playwright check --assert-unique-test-ids` and
`--assert-unique-html-ids` for project-wide uniqueness. The lint rule is
file-local.

`ts-no-export-renaming` is strict by default. To migrate projects that only want
this rule in specific source roots or that intentionally expose default
re-exports as named public APIs, configure it explicitly:

```js
{
  "no-mistakes/ts-no-export-renaming": ["error", {
    includePathPatterns: ["^backend/"],
    allowDefaultReExports: true
  }]
}
```

`nextjs-no-manual-script-tags` allows vetted inline boot scripts by static id or
id pattern while continuing to report other raw `<script>` tags:

```js
{
  "no-mistakes/nextjs-no-manual-script-tags": ["error", {
    allowInlineScriptIds: ["theme-init"]
  }]
}
```
