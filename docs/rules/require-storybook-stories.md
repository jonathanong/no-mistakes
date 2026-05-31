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
