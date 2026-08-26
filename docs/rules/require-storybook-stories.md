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

```tsx
export const Primary = meta.story({ component: UserCard });
```

## Counterexample

```tsx
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
