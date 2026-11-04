# config resolve

Print the effective configuration after discovery, Next.js app resolution, and
test-plan trigger expansion. Use this to snapshot consumer config instead of
string-locking YAML.

## Usage

```bash
no-mistakes config resolve [--root <dir>] [--config <path>] [--view resolved|declared]
```

## Options

| Flag | Description |
|------|-------------|
| `--root` | Project root directory (default: current directory). |
| `--config` | Select exactly one config file; automatic discovery merges the root file and `.no-mistakes/` fragments. |
| `--view` | `resolved` (default) keeps the existing expanded report; `declared` prints the source-level merged config. |

Output is compact serde JSON plus a trailing newline. Named Vitest `fullSuiteTriggers` list entries appear
under `vitestFullSuiteTriggers` with `source: "triggers"`. Deprecated
project-keyed Vitest triggers use `source: "projects"`. The additive
`fullSuiteTriggers` array repeats those entries under `framework: "vitest"` and
includes the same named/project triggers for every other test-plan framework.
Empty frameworks are omitted. Boolean project-keyed triggers expand to the
project's `include` patterns, or `<root>/**` when include is unset. Each
structured trigger includes its effective `includeChangedTests` boolean;
legacy triggers omit the field. Each
`playwright.apps` entry includes the effective `rewrites` and `ignoreRoutes`
after per-project overrides and inheritance from the bound frontend app or
top-level Playwright settings. Per-app `frontendRoot` and `selectorRoots`
honor top-level Playwright defaults before the bound app, matching analysis.

## Declared view

```bash
no-mistakes config resolve --root . --view declared
```

The declared view returns the merged source object directly, with canonical
configuration aliases. It preserves declared values and list entries without
adding defaults, expanding paths, or materializing inherited rule settings.
A repository with no config returns `{}`. It is suitable for readers that find
rules by `rule` and `name` or compare declared entries with deep equality.

The root file is processed first, followed by fragments in sorted relative-path
order. Lists concatenate in that order; this order is part of the public contract.
Relative paths remain repository-root-relative strings. `--config <file>` selects
exactly that file, without fragments. Both views print compact JSON and a newline.
The equivalent async Node API is `resolveConfig({ root: ".", view: "declared" })`.
