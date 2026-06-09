# `no-mistakes react usages`

Map where a React component is used before changing its API. Given a component
file (or a specific exported symbol), it reports every JSX callsite that renders
it, the props passed at each site, the stories and tests that import it, and the
prop type names the component file exports.

```sh
no-mistakes react usages app/components/button.tsx#Button --format json
```

## Target

The first argument is the component to look up:

- `app/components/button.tsx` — match every exported component in the file.
- `app/components/button.tsx#Button` — match only the `Button` export. Use
  `#default` to target a default export.

The path is resolved relative to `--root`.

## Options

- `--scan <glob>` (repeatable) — limit which files are scanned for callsites.
  Defaults to the same TS/JS universe as `react analyze` (root globs, falling
  back to the configured `frontendRoot`).
- `--include stories,tests,props` — emit only the named sections. Callsites are
  always included. Omit the flag to emit every section.
- `--format json|yml|md|paths|human` — `paths` prints the deduplicated callsite,
  story, and test file paths for command substitution.

## Output

```json
{
  "target": { "file": "app/components/button.tsx", "symbol": "Button" },
  "callsites": [
    {
      "file": "app/pages/home.tsx",
      "line": 4,
      "component": "Button",
      "props": ["variant", "onClick"],
      "hasSpread": false
    }
  ],
  "stories": ["app/components/button.stories.tsx"],
  "tests": ["app/components/button.test.tsx"],
  "propTypes": ["ButtonProps", "ButtonVariant"]
}
```

- `props` lists the named JSX attributes passed at the callsite, in source order.
- `hasSpread` is `true` when the callsite spreads props (`{...rest}`), so `props`
  may be incomplete.
- `stories` / `tests` list files that import the target, classified by filename
  (`*.stories.*`, `*.test.*` / `*.spec.*`).
- Each section is omitted when not requested via `--include`.

Node API: `reactUsages({ target, ... })`.
