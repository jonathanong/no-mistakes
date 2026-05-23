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
    "no-mistakes/react-no-nullish-react-node": "error"
  }
}
```

### Rules

| Rule | Purpose |
| --- | --- |
| `no-mistakes/playwright-literals` | Requires JSX test IDs and `getByTestId()` arguments to be static. |
| `no-mistakes/playwright-defaults` | Requires prop-passed test IDs to have string-literal defaults. |
| `no-mistakes/playwright-unique` | Requires literal test IDs to be unique within a file. |
| `no-mistakes/playwright-no-empty` | Disallows empty literal test IDs. |
| `no-mistakes/playwright-consistent-attribute` | Requires one canonical test ID attribute. |
| `no-mistakes/playwright-require-interactive-test-id` | Requires test IDs on interactive JSX elements. |
| `no-mistakes/playwright-prefer-get-by-test-id` | Reports exact CSS test ID selectors passed to Playwright APIs. |
| `no-mistakes/playwright-naming-convention` | Requires literal test IDs to match a regex. |
| `no-mistakes/nextjs-static-fetch-url` | Requires `fetch()` URL arguments to be string literals or expression-free templates. |
| `no-mistakes/nextjs-static-fetch-method` | Requires `fetch()` `method` options to be string literals or expression-free templates. |
| `no-mistakes/ts-no-export-renaming` | Disallows value export aliases such as `export { X as Y }`. |
| `no-mistakes/ts-no-function-aliases` | Disallows wrappers that only forward to another function. |
| `no-mistakes/react-no-nullish-react-node` | Disallows `??` on explicitly typed ReactNode values. |

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

Use `no-mistakes playwright check --assert-unique-test-ids` and
`--assert-unique-html-ids` for project-wide uniqueness. The lint rule is
file-local.
