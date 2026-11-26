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

| Preset                           | Contents                                                                                                                                                                |
| -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `noMistakes.configs.recommended` | Static fetches, direct TypeScript APIs and const identities, basic selector safety, no property deletion, no exported never placeholders, and ReactNode nullish safety. |
| `noMistakes.configs.strict`      | Recommended plus stricter Next.js, Playwright, React, test-state, mock-file, delayed-rejection, and array-await rules.                                                  |

## Editor suggestions

Only two rules expose editor suggestions. `async-call-disposition` offers a
`void` prefix for a bare configured promise; `async-try-catch-return-await`
offers `await` before a returned promise expression inside `try` when that
`catch` calls a configured handler. Neither suggestion
decides whether detaching or catching the work is semantically correct, so
review it before applying.

## Rule options

Rules omitted below have no options. Per-rule pages show valid and invalid code
and point to related rules.

### `async-call-disposition`

`targets` is an array whose objects may set `sourceSpecifierPatterns?: string[]`
and `calleeNamePatterns?: string[]`.

`awaitingConsumers` optionally declares imported APIs that adopt returned callback
promises. Entries require `sourceSpecifierPatterns`, `calleeNamePatterns`, and
at least one callback contract: `callbackArgumentIndexes` (distinct nonnegative
zero-based integers), or `callbackArgumentProperties` (objects with an
`argumentIndex` and distinct `propertyNames` for fresh dependency objects). Only
those callback slots or properties gain the trusted return disposition. Inline callbacks, unchanged
`const` callback bindings, and function declarations with no reassignment qualify;
every call argument use of a named callback must adopt its returned promise.
Stable same-file consumers can also prove adoption by awaiting every invocation
of their callback parameter. Resolvable named imports prove that adoption from
the implementation, including in Oxlint `--type-aware` mode, which does not
expose a TypeScript program. Without that program, `export default`, `export *`,
and `module.exports` stay unproven. A `require()` consumer stays unproven in
either mode. Missing bodies, unresolved imports, `noResolve`, writes, and
escaping implementations stay unproven. Unknown consumers keep reporting.

The cross-file source proof requires the optional `typescript` peer, version 5
or later. Install it with `npm install --save-dev typescript`.
`@typescript-eslint/parser` is optional: when absent, the proof parses supported
source syntax with `ts.createSourceFile`, without constructing a TypeScript
program. In TypeScript-only mode, an `enum`, a `class`, labels, and the other
cases in
`test-cases/eslint-plugin/async-call-disposition/fixture/optional-peers/unsupported/`
stay unproven, while the same consumer is silent when the parser is installed.
With no TypeScript peer, missing or incompatible tooling, or an unsupported
compiler version, dependent calls retain conservative findings. Same-file
proofs and configured contracts still work without either optional peer.

### `async-try-catch-return-await`

`handlers` is an array whose objects may set `sourceSpecifierPatterns?: string[]`
and `calleeNamePatterns?: string[]`.

### `require-options-on-imported-call`

`targets` is an array whose objects set `optionsPosition` (one-based),
`requiredProperties: string[]`, and one selector kind. Imported targets set
`sourceSpecifierPatterns: string[]` and `calleeNamePatterns: string[]`. Global
targets set `globalNamePatterns: string[]`. Receiver-shaped targets set
`calleeNamePatterns` and `receiverPatterns: string[]` (`*` is one dotted
segment, `**` any prefix). A pattern that starts with `/` is a wrapped regex
and cannot contain an unescaped `/` (`/^@acme\/http$/` is valid;
`/^@acme/http$/` and `/abs/**` are rejected). Optional
`propertyMatch?: "any" | "all"` defaults to `"any"`. Optional per-target
`message` or top-level `message` replaces the report text.
`checkedPathPatterns?: string[]` limits files (empty checks every
file) and `allowedPathPatterns?: string[]` skips matches. Empty `targets`
disables the rule. A required property whose statically visible value is
definitely `undefined` does not count. A later spread does not erase an
earlier explicit key.

### `module-mock-boundary`

The schema accepts an object: `internalSpecifiers?: string[]`,
`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`,
`requireLiteralSpecifiers?: boolean` (default `true`),
`baseline?: [string, string, number][]`, `integrationExports?: object`
(including `sourcePathTemplates` and `reexportExtensions`), and
`unanalyzable?: "finding" | "ignore"` (default `"finding"`).

### `module-mock-preserve-exports`

The schema accepts an object: `internalSpecifiers?: string[]`,
`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`, and
`baseline?: [string, string][]`, and
`unanalyzable?: "finding" | "ignore"` (default `"finding"`).

### `nextjs-no-manual-script-tags`

`allowInlineScriptIds?: string[]` and `allowInlineScriptIdPatterns?: string[]`
exempt a raw `<script>` only when it also has `dangerouslySetInnerHTML`.
Both default to no exemptions. Literal `type="application/ld+json"` and
`type={"application/ld+json"}` tags are allowed by default, including an `as`
or `satisfies` wrapper around that literal or an allowed `id`. A spread after the type is not exempt.
`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`. Outside `app`
and `pages`, a non-static `require`, `import()`, or `import = require` reports
on each raw `<script>`. A static `require("next/…")` or
`import Name = require("next/…")` counts as a Next.js file.

Set `structuredDataComponent?: string` to recommend a project component for
JSON-LD, or `structuredDataMessage?: string` for a custom diagnostic. Both
accept nonempty strings; either enables raw JSON-LD reports, and the custom
message wins when both are set. Inline-id exemptions do not bypass this
opt-in. Scope adoption with flat-config `files` and `ignores`, including an
ignore for the wrapper implementation. See the
[rule options and example](eslint-rules/nextjs-no-manual-script-tags.md#options).

### `no-banned-import-outside-allowed-paths`

`checkedPathPatterns?: string[]`, `allowedPathPatterns?: string[]`, and
`bannedImports` is an array of objects with `module: string` and
`names: string[]`. `"default"` in `names` denotes direct default-export calls,
including `.default()`.

### `no-global-fetch-outside-helper`

`checkedPathPatterns?: string[]` and `allowedPathPatterns?: string[]`.

### `no-object-mutation`

Deprecated with no replacement; existing configured findings are unchanged. See
the [migration guide](migrations/deprecated-surfaces.md).

Opt-in. Not enabled by `configs.recommended` or `configs.strict`. `module`,
`imported`, `parameters`, and `calls` default to `true`. `allowThis` defaults
to `true` (class `this` writes are allowed). `allow` is a `string[]` of root
names or dotted paths.

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
`pattern?: string` (default `^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$`).

### `playwright-no-browser-new-page`

Opt-in. Not enabled by `configs.recommended` or `configs.strict`.
`fixtureName?: string` defaults to `"page"` and names the shared page fixture
recommended in diagnostics. `browserFixtureName?: string` defaults to
`"browser"`. `registrationPackages?: string[]` defaults to `[]` and admits
additional modules exporting Playwright `test` and `mergeTests`.
`unanalyzable?: "finding" | "ignore"` defaults to `"finding"` so uncertain
fixture, browser, or callback shapes report instead of silently passing.
`message?: string` customizes definite `browser.newPage()` diagnostics. The
plugin exports `PlaywrightNoBrowserNewPageOptions` and
`PlaywrightRuleUnanalyzable`. `PlaywrightNoBrowserNewPageUnanalyzable` is a
deprecated alias of that type and is removed in 0.111.0. See
[the rule reference](eslint-rules/playwright-no-browser-new-page.md).

### `playwright-no-hoisted-unique-token`

`tokenFactories: string[]`; no default — the rule is inert until configured.

### `playwright-no-raw-scroll-pagination`

`cursorParams?: string[]` (default `["after", "cursor"]`),
`scrollHelper?: string` (default `""`), and optional
`sentinelLocatorNames?: string[]` and `sentinelTestIds?: string[]` to check
`scrollIntoViewIfNeeded()` on configured pagination sentinels.
See [the rule reference](eslint-rules/playwright-no-raw-scroll-pagination.md).

### `playwright-no-swallowed-visibility`

Opt-in. Not enabled by `configs.recommended` or `configs.strict`. `methods`
replaces the default `isVisible`, `isHidden`, `isEnabled`, and `isChecked`
list. `message` replaces the report text. File scope stays on the flat-config
`files` glob. See [the rule reference](eslint-rules/playwright-no-swallowed-visibility.md).

### `playwright-require-exported-component-attribute`

`attributes?: string[]` (default `["data-pw"]`), `componentNamePattern?: string`,
`components?: string[]`, `ignoreComponents?: string[]`, `wrappers?: string[]`,
`allowSpreadAttributes?: boolean` (default `false`),
`exportTypes?: ("named" | "default")[]`, and `checkAnonymousDefault?: boolean`.

### `playwright-require-interactive-test-id`

`selectorAttributes?: string[]` (default `["data-testid", "data-pw"]`) and
`interactiveComponents?: string[]`; component entries may be exact names or
`/regex/` strings. `slotProps?: string[]` defaults to `["asChild"]`; `[]` disables
slot recognition. A configured slot component's literal selector covers one
direct JSX child. Missing or nonliteral slot selectors are reported on the parent.

### `postgres-cursor-call-contract`

`modules: string[]`, `executors: string[]`, `include?: string[]`,
`exclude?: string[]`, `includeFiles?: string[]`, `annotation?: string`,
`sqlTagModules?: string[]`, and `trustedSqlTags?: { module: string, name: string }[]`
(default empty). Empty `modules` or `executors` disables the rule;
`include` defaults to `**/*.{ts,mts,tsx,js,mjs}` and `sqlTagModules` to
`["sql-template-strings"]`. `trustedSqlTags` trusts a named import of `name` from
`module` or a subpath of `module`, including a rename. A default import is not
enough, and a shadowed or rebound local fails closed.

The check rule `postgres-require-query-annotation` accepts any tagged template
at `effort: low` when the first quasi begins with a literal named block comment.
The tag's import form and identity are not checked. In that rule only,
`trustedSqlTags` is deprecated and ignored at low effort, with a configuration
warning; its max-effort behavior and use by other SQL rules are unchanged. See
the [rule reference](rules/postgres-require-query-annotation.md#effort).

### `postgres-no-async-sql-builder`

Opt-in and inert until configured. Not in `configs.recommended` or
`configs.strict`. No autofix and no editor suggestion. `builders` is an array
of `{ names?: string[], callees?: string[], source?: string }`. `names` selects
async functions by local name. `callees` selects async functions that return
those calls or tagged templates. `source`, when set, limits `callees` to a
named import or namespace member from that exact module. The rule tells you to
remove `async` only when every return is a judged SQL string, a module-level
`const` string or no-substitution template, or a configured tag
(not a call such as `sql(id)`, and not a tag placeholder such as `${id.x}`)
and the visible contract allows that synchronous return. Otherwise it
abstains, including for a class member, an interface member, or an object
property. To get a judged report, move the builder to a local non-exported
`async function` that returns a literal, a module-level `const` string, or a
configured tag. It also abstains when a caller parameter or bound local is untyped,
`Promise`-typed, `any`, `unknown`, `object`, a Promise-compatible type
literal, or unresolved, and when the result is only tested for truthiness.
See
[the rule reference](eslint-rules/postgres-no-async-sql-builder.md).

### `postgres-no-correlated-sql-builder`

Opt-in and inert until configured. Not in `configs.recommended` or
`configs.strict`. No autofix and no editor suggestion. `builderMethods` is the
list of builder method names, and `tagNames` is the list of SQL tag
identifiers. Both must be non-empty. A repeated `if` that appends a string,
untagged template, or configured tag to one builder for one selector suggests
one switch that returns a complete statement and throws in `default`.
See
[the rule reference](eslint-rules/postgres-no-correlated-sql-builder.md).

### `postgres-no-executor-adapter`

`importSpecifier?: string` (default empty),
`executorNames?: string[]` (default empty without a module; `["query", "read", "write"]` with a module),
`executorFactoryNames?: string[]` and `executorTypeNames?: string[]` (default empty;
scoped executors that also match imports from `importSpecifier` subpaths and relative imports that resolve into that package, see the [migration notes](migrations/explicit-postgres-executors.md#scoped-executors)),
`trustedSqlTags?: { module: string, name: string }[]` (default empty; tags whose
templates expose SQL in `"forwarding-only"` mode), and `mode?: "all" |
"forwarding-only"` (default `"all"`; that mode reports a call that omits an
optional field, such as `put(tx, { id: row.id })`. The opt-in mode reports
forwarding callbacks and hidden SQL paths, and treats that omission as
undefined). See the [rule reference](eslint-rules/postgres-no-executor-adapter.md).

`nonWritingCallees?: { module: string, export: string }[]` defaults to empty.
Each `module` and `export` pair trusts the exact named import or `export: "default"`
import as non-writing in typed forwarding-only proofs. Stable typed assertion
predicates also preserve otherwise proven bind-value projections. Unknown callees,
unsafe arguments, captures, and writes remain unproven; see the rule reference for
the trust boundary.

### `postgres-no-manual-transaction`

`importSpecifier?: string` (default empty),
`executorNames?: string[]` (default empty without a module; `["query", "read", "write"]` with a module),
`executorFactoryNames?: string[]` and `executorTypeNames?: string[]` (default empty;
scoped executors that also match imports from `importSpecifier` subpaths and relative imports that resolve into that package, see the [migration notes](migrations/explicit-postgres-executors.md#scoped-executors)),
`trustedSqlTags?: { module: string, name: string }[]` (default empty; accepted so a
shared executor configuration validates; this rule still reads every tagged template), and
`owners?: string[]`.

### `postgres-no-nonliteral-sql-registry-key`

Opt-in. Not in `configs.recommended` or `configs.strict`. No autofix and no
editor suggestion. `importSpecifier` selects the executor module.
`executorNames` defaults to `["query", "read", "write"]` when that module is
set. `executorFactoryNames` and `executorTypeNames` also identify scoped
receivers; both default to `[]`. `trustedSqlTags` accepts shared `module` /
`name` entries and defaults to `[]`. With a module, `.query` needs a proven
receiver. A non-literal key into a `const` object of identifier strings, used as
that executor's SQL argument, suggests a switch with one `return` per entry
and a throwing `default`. See
[the rule reference](eslint-rules/postgres-no-nonliteral-sql-registry-key.md).

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

### `postgres-pool-requires-error-listener`

Opt-in. `extraModules?: string[]` defaults to `[]` and adds exact compatible
module names to the built-in `pg` module. A pool needs a same-module
`on("error", ...)`, `once`, or `addListener` registration on its own binding or
class member. Both ECMAScript imports and literal CommonJS `require` calls are
supported. See [the rule reference](eslint-rules/postgres-pool-requires-error-listener.md)
for supported patterns and fixes.

### `react-no-nullish-react-node`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`. A ReactNode
import that cannot be resolved reports instead of passing silently.

### `react-no-use-promise-resolve`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`. A
`use(Promise.resolve(...))` call whose import cannot be resolved reports
instead of passing silently. `require("react")` and
`import React = require("react")` are runtime `use` bindings.

### `server-require-nullable-fetch-wrapper`

`includePathPatterns?: string[]`, `excludePathPatterns?: string[]`,
`getterCalleePatterns: string[]`, `requiredWrapperCallee: string`,
`nullableReturnTypeNames?: string[]`, `inferNullableFromTopLevelEntityPath?: boolean`
(default `false`), `topLevelEntityPathPatterns?: string[]`, and
`unanalyzable?: "finding" | "ignore"` (default `"finding"`).

### `test-no-fixed-temp-paths`

Opt-in. `unanalyzable?: "finding" | "ignore"` defaults to `"finding"` when an
`fs` binding comes from a non-static `require`. Reports fixed temporary paths at known Node
filesystem or process sinks. A local `const` alias can supply a path argument
or the root of a `path.join` or `path.resolve` call. Unique-directory prefixes,
including disposable variants, are allowed. Files that mock the filesystem
with `vi.mock` or `jest.mock` are exempt. See
[the rule reference](eslint-rules/test-no-fixed-temp-paths.md).

### `test-no-shared-state`

`allowBeforeAllAssignments?: boolean` (default `false`) and
`unanalyzable?: "finding" | "ignore"` (default `"finding"`).

### `test-no-skips`

Enable this single-file rule in a flat-config block with explicit test `files`
globs. It reports Vitest/Playwright skip, conditional, focus, pending, and fixme
modifiers, bound Vitest context `skip()` calls, and Playwright `skip` / `fixme`
declaration annotations. `allow` defaults to `[]` and accepts `skip`,
`skipIf`, `runIf`, `only`, `todo`, and `fixme`. Same-file `.extend()` fixture
wrappers stay bound; `testModules` names import specifiers whose exported
`test` is Playwright's, matching the specifier as written.
`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`.
See [the rule reference](eslint-rules/test-no-skips.md) for import matching,
table callbacks, configuration, and `no-mistakes` suppression directives.

### `test-no-typed-enqueue-fake`

Opt-in. `singleExportPattern?: string` matches the imported export name with a
regular expression. `moduleGlobs?: string[]` selects import specifiers and
narrows the export pattern when both are configured. The rule reports Vitest
and Jest mock constructors and mock wrapper types (`Mock<...>` and
`MockedFunction<...>`) that copy the type of a matching import, including
`typeof` on a namespace member and `Parameters<typeof enqueueEmail>`. It also
reports `vi.mocked` and `jest.mocked` casts of matching imports.
`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`.
See [the rule reference](eslint-rules/test-no-typed-enqueue-fake.md).

### `test-no-wall-time-assertions`

Opt-in. `allowedPaths?: string[]` defaults to `[]` and exempts benchmark paths
using globs relative to the ESLint working directory.
`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`. Numeric bounds on elapsed
`Date.now()` or `performance.now()` values are reported in test filenames;
files that call Vitest or Jest fake timers are exempt. See
[the rule reference](eslint-rules/test-no-wall-time-assertions.md).

### `ts-no-const-aliases`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"`. Differently
named `const` aliases and value import-equals aliases are reported. A
qualified or `require()` import-equals is unanalyzable instead of a silent
pass. See [the rule reference](eslint-rules/ts-no-const-aliases.md).

### `ts-no-export-renaming`

`allowDefaultReExports?: boolean` (default `false`),
`includePathPatterns?: string[]`, and
`unanalyzable?: "finding" | "ignore"` (default `"finding"`). `export import`
and CommonJS `exports.Name` / `module.exports.Name` assignments are renames
when the names differ. A right-hand side that is not one identifier is
unanalyzable instead of a silent pass. See
[the rule reference](eslint-rules/ts-no-export-renaming.md).

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

### `playwright-test-timeout-cap`

Opt-in bounded Playwright test deadline policy. Options: `max`, `fixtureMax`,
`unknownValues`, `configUnknownValues`, `registrationPackages`, `exportRoles`,
`configFiles`, and `banOverrides`.
`banOverrides` defaults to false. Set it to true to report every admitted test
deadline override, including values within the cap and bare `setTimeout` or
`slow` references, including escapes through test info. `describe.configure({ timeout })`
is an override. A `timeout` on test details or a hook has no effect.
`test.configure` is not a Playwright API and is not reported. The ban does not
cover assertion timeouts, `page.waitFor*` timeouts, or `test.step` timeouts.
`test.slow()` counts because it changes the deadline; a statically false
condition does not. Project configuration timeout fields stay on the numeric cap.
The plugin exports
`PlaywrightTestTimeoutCapOptions` for TypeScript configuration.
`configUnknownValues: "defer-to-runner"` delegates only unresolved configuration
values to a separate effective runner policy; its default `"inherit"` keeps
`unknownValues` behavior. Known invalid values and checks outside configuration retain their current behavior.
Reviewed helper export roles distinguish assertions from opaque registrars;
builtin SDK and known registrar identities cannot be overridden. Under finding
policy, `effectiveSlot` reports unproved inherited registration deadlines/latches,
even when subsequent local slow operations are legal. This is not canonical
cross-file owner proof. See [the rule reference](eslint-rules/playwright-test-timeout-cap.md).

### `no-vitest-sequential`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"` when a sequential
registration's Vitest binding comes from a non-static `require`. A static
`require("vitest")` and `import = require("vitest")` are runners for `test`,
`it`, and `describe`. Vitest's default import is not a runner. See
[the rule reference](eslint-rules/no-vitest-sequential.md).

### `vitest-mock-test-file-naming`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"` when a module
mock's `vi` or `jest` binding comes from a non-static `require`. See
[the rule reference](eslint-rules/vitest-mock-test-file-naming.md).

### `test-no-delayed-rejects`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"` when a
`.rejects` matcher's `expect` comes from a non-static `require`. See
[the rule reference](eslint-rules/test-no-delayed-rejects.md).

### `test-no-error-message-matching`

`unanalyzable?: "finding" | "ignore"` defaults to `"finding"` when an
assertion's `expect` comes from a non-static `require`. See
[the rule reference](eslint-rules/test-no-error-message-matching.md).

### `vitest-timeout-cap`

Enable this single-file ESLint/Oxlint rule with explicit `files` globs for
Vitest configs/projects and test files. `defaultMax` defaults to `5000` ms for
config/project `testTimeout` and `hookTimeout`; `overrideMax` defaults to
`30000` ms for test/suite/hook overrides and `vi.setConfig` timeout defaults.
With `requireReason: true` (the default), overrides above `defaultMax` require an adjacent
`// no-mistakes-timeout-reason: <specific reason>` comment. Empty and placeholder
reasons report; a reason never bypasses either cap or unknown-value policy.
Set `requireReason: false` for a staged rollout that keeps cap enforcement while
disabling only justification findings.
`configRoot` explicitly admits plain standalone configuration exports; enable it
only for actual configuration-file scopes. Both caps must be positive finite numbers. `unknownValues` defaults to
`ignore`; set `finding` to require statically recoverable timeout values. A
non-static `require` of Vitest is one of those unresolved values.
`configUnknownValues` defaults to `"inherit"`; `"defer-to-runner"` delegates only
unresolved configuration values to a separate effective runner policy without
changing known-invalid findings or those outside configuration.
Same-module constants, aliases, config callbacks with a single return,
object/array spreads and effective `mergeConfig` precedence are supported.
See [the rule reference](eslint-rules/vitest-timeout-cap.md) for limits and
`no-mistakes` file/line/next-line suppressions.

### `gh-api-argument-safety`

Opt-in literal GitHub CLI API argv checks. `wrappers` entries select `callee`,
`argumentPosition`, and `includesApi`; `factoryWrappers` entries select a
`factory`, `method`, `argumentPosition`, and `includesApi` on an immediate
`factory(...).method([...])` receiver; `endpointMessage` and `methodMessage`
customize diagnostics. See [the rule reference](eslint-rules/gh-api-argument-safety.md).

### `no-argv-main-guard`

Opt-in, scope-aware main-module guard lint. Its optional `message` customizes
entrypoint guidance; see [the rule reference](eslint-rules/no-argv-main-guard.md).

### `no-locale-less-date-format`

Opt-in locale-argument checks for formatting methods and constructors. `methods`
replaces the default `toLocaleString`, `toLocaleDateString`, and
`toLocaleTimeString` names. Selecting `toLocaleString` also covers Number and
custom-object methods. `constructors` defaults to an empty list and selects
global members such as `Intl.DateTimeFormat` for both calls and `new`
expressions. `allowUndefinedLocale` defaults to `false` and permits explicit
undefined values when enabled, while missing arguments still report.
`unknownValues` defaults to `"ignore"`; `"finding"` reports spread first
arguments, computed members on configured constructor namespaces, and dynamic
`with`/direct-`eval` scopes. It reports `unanalyzable` for an omitted locale
when a configured constructor namespace comes from an unread import or
`require()`. Supports `no-mistakes-disable-*` directives.
Use flat-config `files`/`ignores` for scope. Direct helper calls are not checked.
The plugin exports `NoLocaleLessDateFormatOptions` for TypeScript configuration.
See [the rule reference](eslint-rules/no-locale-less-date-format.md).

### `no-comment-phrases`

Opt-in comment-phrase lint. `patterns` entries set `pattern`, optional `flags`,
and an optional scoped `message`; a top-level `message` is the fallback.
`requireSuppressionReason` defaults off and, when true, requires a nonempty
reason on this rule's `no-mistakes-disable-*` directives only.
See [the rule reference](eslint-rules/no-comment-phrases.md).

### `forbidden-member-chains`

Opt-in bans for configured static member chains. Each `chains` entry sets
`path`, an optional `receiver` (`*` or a root name), and `message`. A malformed
`path` is a schema error. A `const` string or number key is that segment. A
computed key the rule cannot read is unanalyzable when it could complete a
configured path. Object destructuring of the final segment matches,
and a named root ignores shadowing. File scope stays on the lint config. See
[the rule reference](eslint-rules/forbidden-member-chains.md).

### `restricted-api-usage`

An opt-in file-local import and call policy. `targets` entries select an exact
`importedNamePattern` (with an exact `source`, or alone for any source), an independent lexical
`callNamePattern`, or both. `message` supplies a local reason and safe rewrite.
`discardedResultOnly: true` limits `callNamePattern` to calls whose result is
discarded (expression statement, `void`, non-final comma operand, awaited or not).
Type-only imports are ignored; imported aliases and namespace calls use lexical
binding identity. Lexical call patterns intentionally also match unrelated
bindings with those names. Strict lexical selectors
(`referenceNamePattern`, `declarationNamePattern`, `propertyNamePattern`,
`destructureNamePattern`, `constructNamePattern` with `constructArgumentCount`) are separate opt-ins for
owners who intend the names themselves to be forbidden. `referenceNamePattern`
matches value reads, not TypeScript declaration slots, type names, or assignment
targets. Identical messages on one node collapse. Invalid expressions fail
during initialization.
See [the rule reference](eslint-rules/restricted-api-usage.md) for file scoping,
exclusions, suppression, and complete examples.

### Explicit client and lifecycle contracts

`no-consecutive-unbatched-reads` accepts `clients`, `imports` selectors with `source` and `imported`, required `readMethods`, and optional `message`. It reports adjacent direct awaited reads on the same selected client binding unless the second references a previous result. See [the rule](eslint-rules/no-consecutive-unbatched-reads.md) for the configured independence contract and static limits.

`require-top-level-registration` accepts required `registrar` (`source`, `imported`) and `callback`, plus optional `message`. Scope owner modules with lint configuration; a genuine runtime imported registrar must receive the callback in a direct module-level expression statement. See [the rule](eslint-rules/require-top-level-registration.md).

Both rules are opt-in and support ESLint and Oxlint, custom messages, and file/line/next-line suppression. Neither is enabled by a preset.

### `no-top-level-await`

An opt-in rule for module-level `await`, `for await`, and `await using`.
`allowInImportMetaMain` defaults to `true` inside a direct main-guard consequent;
set it to `false` to report those too. `message` customizes the diagnostic. See [options and examples](eslint-rules/no-top-level-await.md).
