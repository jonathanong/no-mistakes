<!-- cspell:ignore bunx cacheDir nameFilters nohup setupFiles testNamePattern unconfigured -->

# `github-actions-vitest-project-selection`

## Why and when

Use this rule when a GitHub Actions workflow should run a Vitest project as a
whole. A positional file, a positional test name, or `--testNamePattern` runs
a subset and can hide failures in the rest of that project. The CI command
should select the project and leave filtering to Vitest's project config.

The rule is opt-in. Listing it is what turns it on. It is off by default and
is not part of a recommended or strict preset. `options.projects` defaults to
none, and an empty list reports nothing.

## What it catches

The rule reads workflow files directly under `.github/workflows/*.{yml,yaml}`
and composite `action.yml` / `action.yaml` files under `.github/actions/`. A
file is judged only when a configured project lists that path or a glob that
matches it. Other files, including a workflow that selects an unconfigured
project, are silent. Invalid YAML is not a finding. A command that contains
`$` or a backtick is not judged.

A Vitest invocation is the `vitest` command, including a path whose last
segment is `vitest`, or `vitest` launched by `pnpm`, `npm`, `npx`, `yarn`,
`bun`, or `bunx` (`exec`, `x`, and `run vitest`). `pnpm run test` is not
Vitest. Leading assignments and the wrappers `sudo`, `command`, `time`,
`nice`, `nohup`, and `env` do not hide the command.

On an invocation that passes `--project` or `--project=<name>` for a
configured project, a positional test file is always a finding. A positional
test name, `-t`, or `--testNamePattern` is a finding only when that project's
`nameFilters` is `reject`. The default is `allow`, which keeps non-file name
filters silent. When several configured projects are selected, a file finding
names the first and a name finding names the first project whose `nameFilters`
is `reject`.

A positional is a test file when it contains `/`, `\`, `*`, `?`, or `[`, or
when it contains a dot and ends in `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`,
`.cjs`, `.mts`, `.cts`, `.vue`, `.svelte`, or `.json`. Mode words `run`,
`watch`, `dev`, `related`, `bench`, `list`, and `init` are not positionals.
Known value flags consume their next argument or an inline `--flag=value`.
That includes `--config`, `--dir`, `--root`/`-r`, `--workspace`, `--exclude`,
`--coverage.include`, `--coverage.exclude`, `--cacheDir`, `--setupFiles`,
`--changed`, `--outputFile`, and `--outputFile.<reporter>`. `--root ./apps/web`
and `--coverage.include='src/**'` are not file filters. `--` ends option
parsing. Flags this rule does not know do not consume the next token.

Flow-style steps that the line scan cannot place are reported on line 1.

## Options

`options.projects` defaults to an empty list. Each entry requires `project`
and a non-empty `workflows` list of repo-relative paths or globs. `nameFilters`
is `allow` or `reject`; the default is `allow`. Duplicate project names, an
empty project name, an empty workflow pattern, an absolute path, a backslash,
or a `..` segment is a configuration error. Unknown option keys are rejected.

```yaml
rules:
  - rule: github-actions-vitest-project-selection
    scope: repository
    options:
      projects:
        - project: widgets
          workflows:
            - .github/workflows/widgets.yml
        - project: items
          workflows:
            - .github/workflows/items.yml
          nameFilters: reject
```

## Valid example

Select the project and leave file selection to that project's Vitest config.
`widgets` is a test name, and this project's `nameFilters` stays at the default
`allow`.

```yaml
# .github/workflows/widgets.yml
name: Widgets
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm exec vitest run --project widgets
      - run: pnpm exec vitest run --project widgets --reporter=dot
      - run: pnpm exec vitest run --project widgets widgets
```

## Counterexample

A positional file runs one test instead of the whole project:

```yaml
# .github/workflows/widgets.yml
name: Widgets
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm exec vitest run --project widgets src/widgets.test.ts
```

## Fix

Run the configured project without a positional file argument.

```yaml
# .github/workflows/widgets.yml
name: Widgets
on: push
jobs:
  unit:
    runs-on: ubuntu-latest
    steps:
      - run: pnpm exec vitest run --project widgets
```

For a project with `nameFilters: reject`, also drop positional test names and
`-t` / `--testNamePattern`, or set `nameFilters` to `allow` when a non-file
filter is intentional.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Invalid YAML and commands that contain `$` or a backtick produce no finding. They are not sent through `options.unanalyzable`. A literal positional file or name on a recognized Vitest command is judged.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-vitest-project-selection`
suppresses the next line. `no-mistakes-disable-line github-actions-vitest-project-selection`
suppresses the same line. `no-mistakes-disable-file github-actions-vitest-project-selection`
in the leading comment block suppresses the file.

```yaml
# .github/workflows/widgets.yml
# no-mistakes-disable-next-line github-actions-vitest-project-selection
- run: pnpm exec vitest run --project widgets src/widgets.test.ts
- run: pnpm exec vitest run --project widgets src/items.test.ts # no-mistakes-disable-line github-actions-vitest-project-selection
```

## Related rules

[`github-actions-test-timeout-literals`](github-actions-test-timeout-literals.md)
rejects timeout literals copied into workflow tests.
[`vitest-project-policy`](vitest-project-policy.md) checks Vitest project
config files. This rule checks GitHub Actions commands that narrow a
configured project.
