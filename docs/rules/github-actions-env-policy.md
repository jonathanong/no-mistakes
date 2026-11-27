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
steps under `runs.steps` of a composite action. A required value written as `${{ }}`
cannot be judged statically, so it is never a required-value mismatch. The rule
reports it as unanalyzable instead (see below). A forbidden name is still a
finding when its value is an expression, because the name alone is the violation.
Numbers and booleans compare as their string forms. A mapping
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

Each `include` entry must be a valid glob. Variable names in `required` and
`forbidden` must be non-blank.

`required` defaults to `{}` and maps a name to a string, number, boolean,
or YAML null. Null is normalized to the empty string. [Required values](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_workflow_files/env_policy.rs#L39-L58).
`forbidden` defaults to `[]`. A name cannot be both required and forbidden;
that is a configuration error, as is a non-scalar required value. With both
empty the rule reports nothing, so it is opt-in.

`unanalyzable` inherits the repository setting when present and otherwise
defaults to `advise`. [Policy precedence](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/config/v2/analysis_policy.rs#L86-L96).

`include` defaults to `.github/workflows/**/*.yml`,
`.github/workflows/**/*.yaml`, `.github/actions/**/action.yml`, and
`.github/actions/**/action.yaml`. A nonempty list replaces those globs.
An empty list uses the defaults. [Include selection](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_workflow_files/env_policy.rs#L109-L114).

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

An unreadable file yields an unanalyzable diagnostic at line 1. Invalid YAML
yields one at the parser's reported line, or at line 1 when the parser has no
location. Set `rule.message` on the application to replace the diagnostic text.
The checker does not evaluate `${{ }}` expressions: a required name whose
value contains `${{` abstains from the mismatch check and records a diagnostic
(for example `env: { BASH_ENV: ${{ vars.SHELL_INIT }} }` or
`${{ matrix.init }}`). A non-mapping, non-null `env:` value at workflow, job, or
step scope (such as `env: ${{ fromJSON(vars.EXTRA_ENV) }}`), non-mapping
workflow documents, `jobs`, job entries, step entries, or composite `runs`,
non-sequence `steps:` values, non-string `jobs` keys, and YAML merge keys
(`<<:`) are also reported as unanalyzable so dynamic or opaque shapes cannot
bypass `forbidden` or `required` rules silently. `options.unanalyzable`
controls these diagnostics (`advise | report |
ignore`, default `advise`): `advise` records a non-failing advisory, `report`
makes it a failing finding, and `ignore` drops it. Write literal mappings,
step sequences, and literal required values to have them verified, or suppress
the line when the expression is safe.

A ` #` inside a quoted key or value is not treated as a comment when locating
finding lines.

## Related rules

[`github-actions-action-input-policy`](github-actions-action-input-policy.md)
applies the same require-or-ban policy to `with:` inputs.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray
byte is still reported.
