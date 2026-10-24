# `require-storybook-stories`

Requires selected exported components to be covered by Storybook stories.

```yaml
rules:
  - rule: require-storybook-stories
    projects: [web]
    options:
      stories: ["stories/**/*.stories.tsx"]
      includeAllReactNamedExports: true
      exclude: ["app/generated/**"]
```

Counterexample: exporting `UserCard` without a reachable story importing it or a
parent that renders it.

Fix: add a story, render through a covered parent, allow colocated tests where
configured, or exclude the component.

## Why and when

Use this rule for component libraries where exported UI is expected to have an
interactive, reviewable Storybook scenario before it is consumed by the app.

## What it catches/requires

Each selected exported component must be reachable from a configured story or
an allowed covered parent. Generated components and intentionally ineligible
exports should be excluded explicitly.

### Incomplete analysis

The shared `unanalyzable` policy controls explicitly included TSX/JSX files
whose prepared TypeScript facts contain a parse error. Such a file produces one
file-level `unanalyzable` diagnostic at line 1; the rule does not guess which
components the file exports or report missing stories for them. This check
applies only when the file matches `options.include`. A file considered only
through `includeAllReactNamedExports` or `includeAllReactDefaultExports` does
not produce this diagnostic.

For example, an included `UserCard.tsx` containing the incomplete declaration
`export function UserCard() { return <article>` is reported as unanalyzable.

The shared `unanalyzable` policy defaults to `report`; rule-level
`options.unanalyzable` overrides the top-level `unanalyzable` setting
([configuration](../configuration/README.md)). `report` places
the diagnostic among failing findings, `advise` places it among advisories,
and `ignore` omits it. Project scope, test-file exclusions,
`ignoreIndexAndPrivateFiles`, `exclude`, generic rule filters, `allowFiles`,
and file suppression continue to apply. The diagnostic is suppressed by
`no-mistakes-disable-file require-storybook-stories` or
`no-mistakes-disable-line require-storybook-stories` on line 1.

This diagnostic exists because malformed included source cannot be analyzed
for component exports, so the rule cannot establish Storybook coverage for
that file. Fix the parse error and use valid TSX/JSX; if the file is
intentionally outside the coverage policy, remove or narrow the `include`
pattern, or use a reasoned `allowFiles` entry.

## Options and defaults

`stories` selects candidate story files; an empty list matches no stories.
`include` and `exclude` are rule-local component path globs and both default to
empty lists. `includeAllReactNamedExports` and
`includeAllReactDefaultExports` both default to `false`; enable either to treat
the corresponding React exports as components without an explicit marker.
`requiredProps` defaults to empty and limits coverage to components whose source
contains one of the listed prop names. `allowComponents` and `allowFiles` map
an exempt component key or file path to its required reason and default empty.
`allowColocatedTests` defaults to `false`. `ignoreIndexAndPrivateFiles` also
defaults to `false`; enable it to skip `index` and underscore-private source
files. Generic rule filters remain separate from these rule-local globs.

## Analysis

### Effort

- Supports `low` and `max` (default `max`). **Max**: follows the component render graph and resolves dynamic and mock boundary imports for transitive coverage. **Low**: proves direct story coverage only and routes transitive coverage and export-kind boundary exemptions to `unanalyzable`. No partner rule is required.

`max` is the default and preserves transitive component coverage and dynamic or
mock boundary analysis. Use the shared `options.effort: low` to check direct
story coverage without building the component render graph or resolving the
project's dynamic and mock boundary imports:

```yaml
rules:
  - rule: require-storybook-stories
    projects: [web]
    options:
      effort: low
      unanalyzable: advise
      stories: ["stories/**/*.stories.tsx"]
      include: ["src/**/*.tsx"]
```

Low retains story-file discovery, story import and re-export resolution,
namespace-import checks, allow-entry validation, and configured colocated-test
coverage. It does not follow a covered parent's render chain. For example:

```tsx
// stories/Parent.stories.tsx
import { Parent } from '../src/Parent';
export default { component: Parent };
export const Primary = {};
// src/Parent.tsx
import { Child } from './Child';
export function Parent() { return <Child />; }
```

Both efforts establish direct coverage for `Parent`. Max can also establish
coverage for `Child`; low treats its coverage as unanalyzable. Add a story that
directly imports `Child`, use an accepted colocated test when enabled, or use max
to follow the render chain.

If the project has no direct story coverage seeds, an explicitly included
component without an accepted colocated test is still definitely missing a
story at low, just as at max. Otherwise an uncovered component may depend on
transitive coverage. Components selected only by export kind also need boundary
analysis to exclude dynamic or mocked modules; low leaves their coverage
unanalyzable rather than guessing that the exemption applies.

The shared `unanalyzable` policy controls these residual diagnostics: `report`
fails (the default), `advise` emits a non-failing advisory, and `ignore` omits
the residual. Definite missing-story findings and namespace/allow-entry findings
remain findings under every policy. Low's definite findings are contained in
max's findings; total report counts can differ when low reports uncertainty
that max resolves clear. No partner check is required because this uncertainty
remains explicit under the selected policy.

Low-only requests also omit dynamic/mock import facts unless another consumer
needs them. Mixed low/max rule applications retain the facts required by max
and reuse the same source inventory and parse.

The same file and line suppression directives suppress low-effort findings and
advisories. Existing parse-error diagnostics continue to follow `unanalyzable`
at both efforts.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), inputs this rule can't prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove (a parse-error file under `include`, and at low effort a child component covered only through a parent's render chain) follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `src/UserCard.tsx:1: unanalyzable (file has a parse error)`.
  - `advise`: a non-failing advisory.
  - `ignore`: omitted.

  Definite violations remain findings under every policy.

## Valid example

```yaml
# .no-mistakes.yml
projects:
  web: {root: .}
tests:
  storybook:
    configs: .storybook/main.ts
rules:
  - rule: require-storybook-stories
    projects: [web]
    options:
      stories: ["stories/**/*.stories.tsx"]
      includeAllReactNamedExports: true
# .storybook/main.ts
export default { stories: [] };
```

```tsx
// src/UserCard.tsx
export function UserCard() {
  return <article>User</article>;
}
```

```tsx
// stories/UserCard.stories.tsx
import { UserCard } from '../src/UserCard';

export default { component: UserCard };
export const Primary = {};
```

## Counterexample

```yaml
# .no-mistakes.yml
projects:
  web: {root: .}
tests:
  storybook:
    configs: .storybook/main.ts
rules:
  - rule: require-storybook-stories
    projects: [web]
    options:
      stories: ["stories/**/*.stories.tsx"]
      includeAllReactNamedExports: true
```

```tsx
// src/UserCard.tsx
export function UserCard() {
  return <article>User</article>;
}
```

No selected story imports or renders `UserCard`.

## Fix

Add a story that imports the component, cover it through a documented parent,
or exclude generated/internal exports with a narrow glob.

## Suppression

Prefer `exclude` for a stable class of generated components. Use
`no-mistakes-disable-file require-storybook-stories` for a deliberate
one-off, with the reason in the comment.

## Related rules

[`required-companion-imports`](required-companion-imports.md) enforces a direct
companion import contract; [`playwright-coverage`](playwright-coverage.md)
covers runtime routes and selectors instead of stories.
