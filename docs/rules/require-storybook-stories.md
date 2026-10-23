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
