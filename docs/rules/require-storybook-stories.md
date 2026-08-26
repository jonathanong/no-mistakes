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

`stories` selects story files; `includeAllReactNamedExports` defaults to `false`;
`exclude` defaults to empty. Colocated test allowances follow the configured
rule options when present.

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
