# ESLint And Oxlint Plugin

Install the plugin, then choose the preset that fits the repository policy:

```sh
npm install --save-dev eslint-plugin-no-mistakes
```

```js
const noMistakes = require("eslint-plugin-no-mistakes");

module.exports = [
  {
    plugins: { "no-mistakes": noMistakes },
    rules: noMistakes.configs.strict.rules,
  },
];
```

Oxlint loads the same ESLint plugin through `jsPlugins`:

```json
{
  "jsPlugins": ["eslint-plugin-no-mistakes"],
  "rules": { "no-mistakes/playwright-literals": "error" }
}
```

## Presets

| Preset                           | Contents                                                                                                           |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `noMistakes.configs.recommended` | Static fetches, direct TypeScript APIs, basic selector safety, no property deletion, and ReactNode nullish safety. |
| `noMistakes.configs.strict`      | Recommended plus stricter Next.js, Playwright, React, test-state, mock-file, and array-await rules.                |

## Editor suggestions

Only two rules expose editor suggestions. `async-call-disposition` offers a
`void` prefix for a bare configured promise; `async-try-catch-return-await`
offers `await` for a direct configured return inside `try`. Neither suggestion
decides whether detaching or catching the work is semantically correct, so
review it before applying.

## Rule options

Rules omitted below have no options. Per-rule pages show valid and invalid code
and point to related rules.

### `async-call-disposition`

`targets` is an array whose objects may set `sourceSpecifierPatterns?: string[]`
and `calleeNamePatterns?: string[]`.

### `async-try-catch-return-await`

`handlers` is an array whose objects may set `sourceSpecifierPatterns?: string[]`
and `calleeNamePatterns?: string[]`.

### `module-mock-boundary`

The schema accepts an object: `internalSpecifiers?: string[]`,
`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`,
`requireLiteralSpecifiers?: boolean` (default `true`),
`baseline?: [string, string, number][]`, and `integrationExports?: object`
(including `sourcePathTemplates` and `reexportExtensions`).

### `module-mock-preserve-exports`

The schema accepts an object: `internalSpecifiers?: string[]`,
`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`, and
`baseline?: [string, string][]`.

### `nextjs-no-manual-script-tags`

`allowInlineScriptIds?: string[]` and `allowInlineScriptIdPatterns?: string[]`;
both default to no exemptions.

### `no-banned-import-outside-allowed-paths`

`checkedPathPatterns?: string[]`, `allowedPathPatterns?: string[]`, and
`bannedImports` is an array of objects with `module: string` and
`names: string[]`. `"default"` in `names` denotes direct default-export calls,
including `.default()`.

### `no-global-fetch-outside-helper`

`checkedPathPatterns?: string[]` and `allowedPathPatterns?: string[]`.

### `playwright-assertion-timeout-cap`

`max?: number`; default `10000` milliseconds.

### `playwright-consistent-attribute`

`selectorAttributes?: string[]` (default `["data-testid", "data-pw"]`) and
`canonicalAttribute?: string` (default `"data-pw"`).

### `playwright-defaults`, `playwright-no-empty`, `playwright-prefer-get-by-test-id`, and `playwright-unique`

`selectorAttributes?: string[]`; default `["data-testid", "data-pw"]`.

### `playwright-literals`

`selectorAttributes?: string[]` (default `["data-testid", "data-pw"]`),
`allowDefaultedProps?: boolean` (default `true`), and
`allowStaticTemplates?: boolean` (default `false`).

### `playwright-naming-convention`

`selectorAttributes?: string[]` (default `["data-testid", "data-pw"]`) and
`pattern?: string` (the plugin's kebab-case pattern by default).

### `playwright-no-hoisted-unique-token`

`tokenFactories: string[]`; no default — the rule is inert until configured.

### `playwright-no-raw-scroll-pagination`

`cursorParams?: string[]` (default `["after", "cursor"]`) and
`scrollHelper?: string` (default `""`; interpolated into the report message
when set).

### `playwright-require-exported-component-attribute`

`attributes?: string[]` (default `["data-pw"]`), `componentNamePattern?: string`,
`components?: string[]`, `ignoreComponents?: string[]`, `wrappers?: string[]`,
`allowSpreadAttributes?: boolean` (default `false`),
`exportTypes?: ("named" | "default")[]`, and `checkAnonymousDefault?: boolean`.

### `playwright-require-interactive-test-id`

`selectorAttributes?: string[]` (default `["data-testid", "data-pw"]`) and
`interactiveComponents?: string[]`; component entries may be exact names or
`/regex/` strings.

### `postgres-cursor-call-contract`

`modules: string[]`, `executors: string[]`, `include?: string[]`,
`exclude?: string[]`, `includeFiles?: string[]`, `annotation?: string`, and
`sqlTagModules?: string[]`. Empty `modules` or `executors` disables the rule;
`include` defaults to `**/*.{ts,mts,tsx,js,mjs}` and `sqlTagModules` to
`["sql-template-strings"]`.

### `postgres-no-manual-transaction`

`importSpecifier?: string` (default `"@data-stores/psql"`),
`executorNames?: string[]` (default `["query", "read", "write"]`), and
`owners?: string[]`.

### `postgres-no-unbounded-query-fanout`

`importSpecifier?: string`, `executorNames?: string[]`, and
`chunkFunctionNames?: string[]` (default `["chunkArray"]`).

### `server-require-nullable-fetch-wrapper`

`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`,
`getterCalleePatterns: string[]`, `requiredWrapperCallee: string`,
`nullableReturnTypeNames?: string[]`, `inferNullableFromTopLevelEntityPath?: boolean`
(default `false`), and `topLevelEntityPathPatterns?: string[]`.

### `test-no-shared-state`

`allowBeforeAllAssignments?: boolean`; default `false`.

### `ts-no-export-renaming`

`allowDefaultReExports?: boolean` (default `false`) and
`includePathPatterns?: string[]`.

### `ts-preserve-null-option-defaults`

`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`,
`optionObjectNames?: string[]`, and `optionObjectNamePatterns?: string[]`.

## Example configuration

```js
module.exports = [
  {
    plugins: { "no-mistakes": noMistakes },
    rules: {
      "no-mistakes/playwright-consistent-attribute": [
        "error",
        { selectorAttributes: ["data-testid", "data-pw"], canonicalAttribute: "data-pw" },
      ],
      "no-mistakes/async-call-disposition": [
        "error",
        {
          targets: [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["/^enqueue/"] }],
        },
      ],
      "no-mistakes/no-global-fetch-outside-helper": [
        "error",
        { checkedPathPatterns: ["web/**"], allowedPathPatterns: ["web/lib/api/**"] },
      ],
    },
  },
];
```

See the [ESLint rule index](eslint-rules/README.md) for behavior, fixes, and
suppression guidance.
