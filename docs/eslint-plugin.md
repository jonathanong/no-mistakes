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

| Preset                           | Contents                                                                                                                                |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `noMistakes.configs.recommended` | Static fetches, direct TypeScript APIs and const identities, basic selector safety, no property deletion, and ReactNode nullish safety. |
| `noMistakes.configs.strict`      | Recommended plus stricter Next.js, Playwright, React, test-state, mock-file, delayed-rejection, and array-await rules.                  |

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

### `require-options-on-imported-call`

`targets` is an array whose objects set `sourceSpecifierPatterns: string[]`,
`calleeNamePatterns: string[]`, `optionsPosition` (one-based),
`requiredProperties: string[]`, and optional `propertyMatch?: "any" | "all"`
(default `"any"`). Empty `targets` disables the rule. A required property
whose statically visible value is definitely `undefined` does not count.

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

### `no-inline-noop-promise-catch`

`checkedPathPatterns?: string[]` and `allowedPathPatterns?: string[]` scope
files (empty `checkedPathPatterns` checks every file).
`allowedCalleeNamePatterns?: string[]` skips matching originating call names.

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
`exclude?: string[]`, `includeFiles?: string[]`, `annotation?: string`,
`sqlTagModules?: string[]`, and `trustedSqlTags?: { module: string, name: string }[]`
(default empty). Empty `modules` or `executors` disables the rule;
`include` defaults to `**/*.{ts,mts,tsx,js,mjs}` and `sqlTagModules` to
`["sql-template-strings"]`. `trustedSqlTags` trusts a named import of `name` from
`module` or a subpath of `module`, including a rename. A default import is not
enough, and a shadowed or rebound local fails closed.

### `postgres-no-manual-transaction`

`importSpecifier?: string` (default empty),
`executorNames?: string[]` (default empty without a module; `["query", "read", "write"]` with a module),
`executorFactoryNames?: string[]` and `executorTypeNames?: string[]` (default empty;
scoped executors that also match imports from `importSpecifier` subpaths and relative imports that resolve into that package, see the [migration notes](migrations/explicit-postgres-executors.md#scoped-executors)),
`trustedSqlTags?: { module: string, name: string }[]` (default empty; accepted so a
shared executor configuration validates; this rule still reads every tagged template), and
`owners?: string[]`.

### `postgres-no-unbounded-query-fanout`

`importSpecifier?: string`, `executorNames?: string[]`,
`executorFactoryNames?: string[]`, `executorTypeNames?: string[]`,
`trustedSqlTags?: { module: string, name: string }[]` (default empty; accepted so a
shared executor configuration validates; this rule still reads every tagged template), and
`chunkFunctionNames?: string[]` (default `["chunkArray"]`).

The Rust-only `reportUnmatchedExecutorNames` option (reports configured executor
names no scanned file imports) is not available in these ESLint rules: they run per
file and cannot know a name never matched anywhere. See the
[migration notes](migrations/explicit-postgres-executors.md#reporting-unmatched-names).

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

Executor import matching has no module default. Set `importSpecifier` explicitly
to your database module to enable default `query`, `read`, and `write` names.
Without a module, only explicit `executorNames` select named imports; explicit
`query` also enables `.query` members. Configured modules retain member matching
with custom executor names. See the [migration notes](migrations/explicit-postgres-executors.md).

### `test-no-skips`

Enable this single-file rule in a flat-config block with explicit test `files`
globs. It reports Vitest/Playwright skip, conditional, focus, pending, and fixme
modifiers and bound Vitest context `skip()` calls. `allow` defaults to `[]` and
accepts `skip`, `skipIf`, `runIf`, `only`, `todo`, and `fixme`.
See [the rule reference](eslint-rules/test-no-skips.md) for import matching,
table callbacks, configuration, and `no-mistakes` suppression directives.

### `playwright-test-timeout-cap`

Opt-in bounded Playwright test deadline policy. Options: `max`, `fixtureMax`,
`unknownValues`, `registrationPackages`, `exportRoles`, and `configFiles`.
Reviewed helper export roles distinguish assertions from opaque registrars;
builtin SDK and known registrar identities cannot be overridden. Under finding
policy, `effectiveSlot` reports unproved inherited registration deadlines/latches,
even when subsequent local slow operations are legal. This is not canonical
cross-file owner proof. See [the rule reference](eslint-rules/playwright-test-timeout-cap.md).

### `vitest-timeout-cap`

Enable this single-file ESLint/Oxlint rule with explicit `files` globs for
Vitest configs/projects and test files. `defaultMax` defaults to `5000` ms for
config/project `testTimeout` and `hookTimeout`; `overrideMax` defaults to
`30000` ms for test/suite/hook overrides and `vi.setConfig` timeout defaults.
`configRoot` explicitly admits plain standalone configuration exports; enable it
only for actual configuration-file scopes. Both caps must be positive finite numbers. `unknownValues` defaults to
`ignore`; set `finding` to require statically recoverable timeout values.
Same-module constants, aliases, config callbacks with a single return,
object/array spreads and effective `mergeConfig` precedence are supported.
See [the rule reference](eslint-rules/vitest-timeout-cap.md) for limits and
`no-mistakes` file/line/next-line suppressions.
