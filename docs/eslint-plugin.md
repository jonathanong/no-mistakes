# ESLint And Oxlint Plugin

The rule reference now lives in [docs/eslint-rules/](eslint-rules/README.md).

Install:

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

ESLint flat config:

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [
  {
    plugins: { "no-mistakes": noMistakes },
    rules: noMistakes.configs.strict.rules,
  },
];
```

Oxlint:

```json
{
  "jsPlugins": ["eslint-plugin-no-mistakes"],
  "rules": {
    "no-mistakes/playwright-literals": "error"
  }
}
```

## Presets

| Preset | Contents |
| --- | --- |
| `noMistakes.configs.recommended` | Default static-safety rules: static fetches, no property deletion, literal Playwright selectors, unique test IDs, ReactNode nullish checks, direct TS exports, and function alias bans. |
| `noMistakes.configs.strict` | Recommended plus stricter Next.js, Playwright, React, test-state, and mock-file naming rules. |

## Rule Options

Rules not listed here have no options.

| Rule | Options |
| --- | --- |
| `playwright-assertion-timeout-cap` | `{ max?: number }`, default `10000`. |
| `playwright-consistent-attribute` | `{ selectorAttributes?: string[], canonicalAttribute?: string }`, defaults `["data-testid", "data-pw"]` and `"data-pw"`. |
| `playwright-defaults` | `{ selectorAttributes?: string[] }`. |
| `playwright-literals` | `{ selectorAttributes?: string[], allowDefaultedProps?: boolean, allowStaticTemplates?: boolean }`; `allowDefaultedProps` defaults to `true`, `allowStaticTemplates` defaults to `false`. |
| `playwright-naming-convention` | `{ selectorAttributes?: string[], pattern?: string }`, default kebab-case pattern. |
| `playwright-no-empty` | `{ selectorAttributes?: string[] }`. |
| `playwright-prefer-get-by-test-id` | `{ selectorAttributes?: string[] }`. |
| `playwright-require-exported-component-attribute` | `{ attributes?: string[], componentNamePattern?: string, components?: string[], ignoreComponents?: string[], wrappers?: string[], allowSpreadAttributes?: boolean, exportTypes?: ("named" \| "default")[], checkAnonymousDefault?: boolean }`. |
| `playwright-require-interactive-test-id` | `{ selectorAttributes?: string[] }`. |
| `playwright-unique` | `{ selectorAttributes?: string[] }`. |
| `nextjs-no-manual-script-tags` | `{ allowInlineScriptIds?: string[], allowInlineScriptIdPatterns?: string[] }`. |
| `test-no-shared-state` | `{ allowBeforeAllAssignments?: boolean }`. |
| `ts-no-export-renaming` | `{ allowDefaultReExports?: boolean, includePathPatterns?: string[] }`. |

```js
module.exports = [
  {
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/playwright-consistent-attribute": [
        "error",
        { selectorAttributes: ["data-testid", "data-pw"], canonicalAttribute: "data-pw" },
      ],
      "no-mistakes/nextjs-no-manual-script-tags": [
        "error",
        { allowInlineScriptIds: ["json-ld"], allowInlineScriptIdPatterns: ["^ld-json-"] },
      ],
      "no-mistakes/ts-no-export-renaming": [
        "error",
        { includePathPatterns: ["^src/"], allowDefaultReExports: true },
      ],
    },
  },
];
```

See [ESLint rule index](eslint-rules/README.md) for per-rule behavior.
