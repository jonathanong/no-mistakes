# `github-actions-env-policy`

## Why and when

`env:` at workflow, job or step scope changes how every shell and tool in its
reach starts. `BASH_ENV` makes each non-interactive `bash` step source a file.
`NODE_OPTIONS` injects options into every Node process. Use this rule when a
repository wants specific values pinned, or specific names kept out entirely.
The checker hardcodes no names. A repository opts in by listing them.

## What it requires

`required` maps an env name to the only value it may have wherever it is set.
`forbidden` lists names that must not appear. Names are case-sensitive. Absence
of a required name is not reported: the rule judges values that are written, so
a step that never sets `BASH_ENV` stays silent.

The policy applies to `env:` at workflow scope, job scope, step scope, and in
steps under `runs.steps` of a composite action. A value written as `${{ }}` is
compared as that text, so it is a mismatch unless the expression text equals the
required value. Numbers and booleans compare as their string forms. A mapping
or list value is a mismatch. Names that appear only inside a `run:` string, and
names that are not configured, are never reported.

## Valid example

```yaml
env:
  BASH_ENV: /dev/null
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: mentions names only in a script
        run: echo "NODE_OPTIONS=--inspect"
      - name: name outside the policy
        env:
          OTHER_OPTIONS: --inspect
        run: echo ok
```

## Counterexample

With `required: { BASH_ENV: /dev/null }` and `forbidden: [NODE_OPTIONS]`:

```yaml
env:
  BASH_ENV: ./setup.sh
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      NODE_OPTIONS: --max-old-space-size=4096
    steps:
      - run: echo hello
```

## Fix

Set the required value and delete forbidden names.

```yaml
env:
  BASH_ENV: /dev/null
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hello
```

## Suppression

Use a YAML comment on the env line to keep. `no-mistakes-disable-line
github-actions-env-policy` applies to the same line,
`no-mistakes-disable-next-line github-actions-env-policy` to the next line, and
`no-mistakes-disable-file github-actions-env-policy` skips the file.

```yaml
steps:
  - run: ./build.sh
    env:
      BASH_ENV: ./setup.sh # no-mistakes-disable-line github-actions-env-policy
```

## Options and defaults

`required` defaults to `{}` and maps a name to a string, number, or boolean.
`forbidden` defaults to `[]`. A name cannot be both required and forbidden;
that is a configuration error, as is a non-scalar required value. With both
empty the rule reports nothing, so it is opt-in.

`include` defaults to `.github/workflows/**/*.yml`,
`.github/workflows/**/*.yaml`, `.github/actions/**/action.yml`, and
`.github/actions/**/action.yaml`. An explicit list replaces that default.

```yaml
rules:
  - rule: github-actions-env-policy
    scope: repository
    options:
      required:
        BASH_ENV: /dev/null
      forbidden:
        - NODE_OPTIONS
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Invalid YAML is skipped and produces no finding. A file that cannot be read is
skipped. The checker does not evaluate `${{ }}` expressions.

## Related rules

[`github-actions-action-input-policy`](github-actions-action-input-policy.md)
applies the same require-or-ban policy to `with:` inputs.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
