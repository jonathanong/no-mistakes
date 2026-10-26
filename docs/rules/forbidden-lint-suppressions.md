# `forbidden-lint-suppressions`

Reject selected lint-suppression directives that disable protected rule IDs.

```yaml
rules:
  - rule: forbidden-lint-suppressions
    scope: repository
    options:
      files: ["src/**/*.{js,jsx,ts,tsx}"]
      ruleIds: [react/purity]
      rulePrefixes: [react-hooks-js/]
      banBlanketDisable: true
      message: "Fix the component's purity or refs violation instead of disabling its rule."
```

## Why and when

Use this repository check when a centrally enforced lint rule must remain active
in selected source files. It also checks oxlint directives when ESLint does not
own those files. Keeping the policy in one configured application avoids
separate scripts that mistake strings or examples for actual comments.

## What it catches

The check reads real comments from the shared TS/JS AST fact pass. It recognizes
line and block comments, comma-separated rule lists, and the same-line and
next-line variants. Quoted rule IDs are matched without their surrounding quotes. Strings, template text, regex literals, JSX text, and JSX attribute
values are not comment directives.

Each finding reports the source file, the directive's line, and one forbidden
rule ID in `target`. A directive that names multiple forbidden IDs produces one
finding per offending ID. A trailing `--` reason does not become a rule ID.

`ruleIds` matches exact strings: `react/purity` does not also select
`react/purity-extra`. `rulePrefixes` matches literal prefixes; configure
`react-hooks-js/` to select that namespace, including its slash. No namespace
separator or wildcard is added automatically.

`files` is required and must contain at least one repository-relative glob. It narrows the rule
application's existing project/include/exclude scope; it never selects a file
outside that scope. Unsupported file extensions are excluded. There is no
implicit whole-repository file selection.

## Options and defaults

| Option              | Meaning and default                                                                                          |
| ------------------- | ------------------------------------------------------------------------------------------------------------ |
| `files`             | Required nonempty list of repository-relative source-file globs.                                             |
| `ruleIds`           | Exact forbidden lint rule IDs; default `[]`.                                                                 |
| `rulePrefixes`      | Literal forbidden lint rule ID prefixes; default `[]`.                                                       |
| `directives`        | Selected directive names; defaults to the six lint forms below.                                              |
| `banBlanketDisable` | Reject selected lint directives with no listed IDs; default `false`.                                         |
| `message`           | Optional custom explanation; by default the diagnostic names the forbidden suppression and how to remove it. |

The default directive names are `eslint-disable`, `eslint-disable-line`,
`eslint-disable-next-line`, `oxlint-disable`, `oxlint-disable-line`, and
`oxlint-disable-next-line`. A configured `directives` list replaces that default; `[]` selects no directives.
An unrelated allowed rule ID does not produce a finding. An empty lint rule
list produces a finding with target `*` only when `banBlanketDisable` is enabled.
That option does not widen `files` or enable this check by itself.

TypeScript's `@ts-ignore` and `@ts-expect-error` are opt-in. For example, this
selection checks those two directives instead of the default lint directives:

```yaml
directives: ["@ts-ignore", "@ts-expect-error"]
```

Each selected TypeScript directive is reported with its directive name as the
finding target. Lint `ruleIds` and `rulePrefixes` do not narrow that selection.

For block comments, TypeScript directives must be on the final content line
before `*/`. A leading JSDoc star is supported:

```ts
/**
 * @ts-ignore */
```

Putting the closing `*/` on a separate following line does not create a
TypeScript directive. Triple-slash forms are supported; adding a fourth slash
does not create one. The finding still points to the directive's line.

## Valid example

With `react/purity` protected, this unrelated suppression remains allowed:

```ts
// eslint-disable-next-line no-console -- logging is intentional
console.log("starting");
const example = "eslint-disable react/purity";
```

Counterexample:

```ts
/* eslint-disable react/purity, react-hooks-js/refs -- temporary workaround */
```

Both configured IDs are reported. A file outside `files` is not checked, even
when it contains the same comment.

## Fix

Remove the forbidden ID from the directive and fix the underlying lint
violation. If the code is intentionally outside this policy, narrow the
configured file scope. Blanket disables can be rewritten to list only allowed
rule IDs. Removing a TypeScript suppression requires fixing the type error or
adjusting the explicit TypeScript directive selection.

There is no automatic fix: deleting a suppression without fixing its cause can
expose an existing lint or type error.

## Suppression

The standard no-mistakes directives apply to this check, including file,
same-line, and next-line suppression. For a reviewed exception:

```ts
// no-mistakes-disable-next-line forbidden-lint-suppressions -- reviewed exception
// eslint-disable-next-line react/purity
```

Use `no-mistakes-disable-file forbidden-lint-suppressions` for a whole-file
exception or `no-mistakes-disable-line forbidden-lint-suppressions` on the
reported line. These are independent of the lint directives being checked.

## Related rules

- [`forbidden-calls`](forbidden-calls.md) restricts calls through configured roots.
- [`require-storybook-stories`](require-storybook-stories.md) requires coverage
  for selected UI components.

## CLI and Node API

Run `no-mistakes check --root . --format json`, or await `check({ root, config })`
from `no-mistakes`. The check uses the existing asynchronous `checkJson` binding
and appears in the ordinary `rules` report. Batched `analyzeProject()` check
reports use the same configured behavior. Import the exported
`ForbiddenLintSuppressionsOptions` and `ForbiddenLintSuppressionDirective` types
when constructing configuration for downstream tools.
