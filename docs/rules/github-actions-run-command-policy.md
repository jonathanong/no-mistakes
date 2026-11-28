<!-- cspell:ignore ccore nohup pwsh XGET -->

# `github-actions-run-command-policy`

## Why and when

Workflow and composite-action `run` scripts invoke CLIs whose safe use depends
on specific flags or argument shapes:

- `gh api` with `-f` or `-F` silently switches to `POST` unless `--method` (or
  `-X`) is passed.
- A hand-built `?key=value` query string inside a `gh api` endpoint mixes
  untrusted or interpolated data with query syntax instead of using structured
  `-f` / `-F` fields and `--method GET`.
- A CI `git commit` should pass `-c core.hooksPath=/dev/null` so checked-out
  repository hooks cannot execute.

This rule is opt-in and has no hardcoded commands. A repository declares the
exact commands, required arguments or argument pairs, forbidden argument
patterns, and messages it wants to enforce.

## What it catches

The rule parses `run` scripts in workflow files directly under
`.github/workflows/*.{yml,yaml}` and composite actions under
`.github/actions/**/action.{yml,yaml}` using the shared shell script parser.
When `options.shellFiles` is configured, it also scans matching standalone shell
scripts with the same parser, policy evaluation, abstentions, and suppressions.

For each configured entry in `options.commands`:

- `command` matches the executable basename (for example `gh` or `/usr/bin/gh`)
  followed by any configured subcommand tokens (for example `api` or `commit`).
  Global flags before a subcommand (such as `git -C packages/app commit` or
  `git -c core.hooksPath=/dev/null commit`) are skipped using `valueFlags` and
  any flag inferred from a two-token argument pair.
- Leading shell wrappers (`if`, `then`, `do`, `env`, `sudo`, `command`, `exec`,
  `nohup`, `time`, `xargs`, and leading `KEY=VALUE` assignments), nested
  `bash -c` / `sh -c` / `zsh -c` literal scripts, and `$(...)` / backtick
  command substitutions are inspected. Single-quoted strings and `$(( ... ))`
  arithmetic expansions are not executed as commands.
- If `whenAnyArgs` is non-empty, the policy applies only when at least one
  listed argument or argument pair is present on the invocation. A dynamic
  word is uncertain when it could expand to a listed flag, including an
  option-shaped word (`tool run "$MODE"` or `tool run "--${MODE}"` with
  `whenAnyArgs: [--write]`). The policy abstains only when that uncertainty
  could still hide a missing requirement or exemption. A requirement that is
  already present stays quiet (`tool run --strict "--${MODE}"` with
  `requireArgs: [--strict]`).
- If `unlessAnyArgs` is non-empty, the policy is skipped when any listed
  argument or argument pair is present (for example `unlessAnyArgs: [graphql]`
  so `gh api graphql -f query=...` stays exempt from requiring `--method`).
  Non-flag single-token matchers match positional arguments only, so values
  consumed by `valueFlags` (such as `-q graphql`) never exempt a command. A
  dynamic word that could expand to a `valueFlags` entry makes a following
  positional exemption uncertain: `tool emit "$OPT" exempt` with
  `valueFlags: [--label]` and `unlessAnyArgs: [exempt]` does not skip the
  policy. An option-shaped dynamic word is treated the same way
  (`gh api "--${FLAG}" graphql` with `valueFlags: [--jq]` and
  `unlessAnyArgs: [graphql]`). Literal `--label exempt` and literal
  `--jq graphql` still consume the following word.
- Every entry in `requireArgs` must be present. If `requireAnyArgs` is
  non-empty, at least one listed alternative must be present. Flag matchers
  skip values consumed by `valueFlags`, accept separate (`--method GET`,
  `-c core.hooksPath=/dev/null`), inline (`--method=GET`,
  `-c=core.hooksPath=/dev/null`), and attached short value flag (`-XGET`,
  `-ccore.hooksPath=/dev/null`) forms, and stop at `--`. Required flags listed
  in `valueFlags` must also have a non-empty value (`--method=`, `-X ""`, and a
  dangling `-X` do not satisfy the requirement). A splitting expansion after
  that `--` cannot supply a flag: `gh api -f x=1 -- $ENDPOINT` still misses
  `--method` or `-X`. A `--` consumed as a `valueFlags` value does not end
  option parsing, so a splitting word after it can still hide a missing flag.
- Every entry in `forbidArgPatterns` tests either all arguments (`scope: any`,
  the default) or positional arguments after the subcommand (`scope:
  positional`, skipping options and values consumed by `valueFlags`).
- Commands that are not configured stay silent.

When the parser encounters a command it cannot statically analyze—such as a
dynamic command name (`$CMD ...` or `$DIR/$TOOL ...`), `eval`, a dynamic
`bash -c "$SCRIPT"`, a dynamic subcommand on a configured base command, an
unquoted word-splitting expansion before `--` that could supply a required
argument, a dynamic value for a required flag (`--method "$METHOD"`), a dynamic positional
that could match an `unlessAnyArgs` exemption when a requirement would
otherwise fail (`gh api -f x=1 "$ENDPOINT"` with `unlessAnyArgs: [graphql]`),
a dynamic word that could expand to a configured flag (`tool run "$MODE"`
or `tool run "--${MODE}"` with `whenAnyArgs: [--write]`; a requirement that
is already present does not abstain), to an inline `flag=value` or attached short
form on a `whenAnyArgs` or `unlessAnyArgs` pair gate (`tool run "$ARG"` with
`whenAnyArgs: [[--mode, write]]`), or to a `valueFlags` entry that would
consume the next word (`tool emit "$OPT" exempt` with `valueFlags: [--label]`,
or `tool run "--label${TAIL}" --skip` with `unlessAnyArgs: [--skip]`),
a dynamic word that could expand to `--` before a later required flag
(`tool run "$STOP" --strict`, including when another flag sits between them),
a configured token whose text equals an expansion's source (`tool run "$REMOTE"`
with `unlessAnyArgs: ["$REMOTE"]`), a command substitution whose `)` sits inside
quotes (`--method="$( : ')' )"`), an escaped closing `)` (`--method="$( : \) )"`),
quotes inside a nested `$(...)`, or an unterminated quote, expansion, or
here-document—it abstains from
reporting a policy violation and records a diagnostic under the configured
uncertainty policy. A variable-prefixed
path whose final path segment is a literal matching no configured policy
`command[0]` and not a known shell (`sh`, `bash`, `zsh`, `dash`, `ksh`, `fish`,
`pwsh`) or wrapper command (`env`, `sudo`, `xargs`, `nohup`, `time`, `exec`,
`command`), such as `$GITHUB_ACTION_PATH/build-images.sh --tag "$TAG"`, abstains
silently without emitting an advisory; any command substitutions inside its
arguments are still inspected.

## Options and defaults

`options.commands` defaults to `[]`, so the rule reports nothing until configured.
`options.include` defaults to `[]`, which selects `.github/workflows/*.{yml,yaml}`
and `.github/actions/**/action.{yml,yaml}`; an explicit glob list narrows the
matched files within those workflow and composite-action paths.
`options.shellFiles` defaults to `[]`, which selects no standalone shell scripts;
an explicit repo-relative glob list (for example `["scripts/**/*.sh"]`) adds
matching standalone shell scripts within the rule's scope (workflow and
composite-action YAML files keep their YAML handling and are not scanned twice).
`options.unanalyzable` accepts `advise`, `report`, or `ignore`. It inherits
the repository-level `unanalyzable` setting when omitted. With neither
setting, the rule uses `advise`. Definite command-policy violations remain
findings in all three modes. [Policy selection](https://github.com/jongleberry/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/filesystem_dispatch/github_actions_run_command_policy/diagnostics.rs#L61-L71).
Invalid option values—including invalid globs in `include` or `shellFiles`,
malformed regexes, or empty command entries—fail early at config load with exit
code 2.

| Option | Default | Behavior |
| --- | --- | --- |
| `commands` | `[]` | List of command policy entries. Each entry requires a non-empty `command` token array and at least one of `requireArgs`, `requireAnyArgs`, or `forbidArgPatterns`. |
| `commands[].command` | Required | Exact command tokens to match, such as `[gh, api]` or `[git, commit]`. |
| `commands[].valueFlags` | `[]` | Flags known to consume the following token when written without `=`. Used to separate flag values from subcommands and positional arguments, and to require non-empty values when a flag is required. Two-token flag pairs in `whenAnyArgs`, `unlessAnyArgs`, `requireArgs`, and `requireAnyArgs` are added automatically. |
| `commands[].whenAnyArgs` | `[]` | Optional guard list of arguments or adjacent token pairs; when non-empty, the policy applies only if at least one matches. |
| `commands[].unlessAnyArgs` | `[]` | Optional exemption list of arguments or adjacent token pairs; when any matches, the policy is skipped. Non-flag entries match positional arguments. |
| `commands[].requireArgs` | `[]` | Arguments or adjacent token pairs that must all be present on every matching command. |
| `commands[].requireAnyArgs` | `[]` | Alternative arguments or adjacent token pairs; at least one must be present. |
| `commands[].forbidArgPatterns` | `[]` | Regex checks with `{ pattern, scope?, message? }`, where `scope` is `any` (default) or `positional`. |
| `commands[].message` | Omitted | Optional policy explanation appended to findings from this entry. |
| `include` | `[]` | Optional repo-relative globs narrowing the checked workflow and action files. |
| `shellFiles` | `[]` | Optional repo-relative globs selecting standalone shell scripts to check with the same command policies. |
| `unanalyzable` | Repository setting, then `advise` | Controls diagnostics for inputs the rule cannot analyze (`advise`, `report`, or `ignore`). |

```yaml
rules:
  - rule: github-actions-run-command-policy
    scope: repository
    options:
      shellFiles:
        - scripts/**/*.sh
      commands:
        - command: [gh, api]
          valueFlags: [-f, -F, --field, --raw-field, -X, --method, -H, --header, -q, --jq]
          whenAnyArgs: [-f, -F, --field, --raw-field]
          unlessAnyArgs: [graphql]
          requireAnyArgs: [--method, -X]
          message: "gh api with -f/-F defaults to POST; pass --method (or -X) explicitly."
        - command: [gh, api]
          valueFlags: [-f, -F, --field, --raw-field, -X, --method, -H, --header, -q, --jq]
          forbidArgPatterns:
            - pattern: '\?'
              scope: positional
          message: "Pass query parameters with -f/-F and --method GET instead of a query string in the endpoint."
        - command: [git, commit]
          valueFlags: [-c, -C, --git-dir, --work-tree]
          requireArgs:
            - [-c, core.hooksPath=/dev/null]
          message: "Run CI commits with -c core.hooksPath=/dev/null so repository hooks cannot execute."
```

## Valid example

With the options above, explicit `--method GET` / `-X GET`, `?` inside a flag
value rather than the endpoint positional, `-c core.hooksPath=/dev/null`, and
commands not in the policy all pass:

```yaml
# .github/workflows/ci.yml
name: Release
on: push
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - run: gh api --method GET -f x=1 repos/my-org/my-repo/labels
      - run: gh api -X GET -F per_page=100 -f "note=why?" repos/my-org/my-repo/labels
      - run: 'git -c core.hooksPath=/dev/null commit -m "chore: update"'
      - run: git status
```

## Counterexample

Missing `--method` with `-f`, a query string in the positional endpoint, and a
commit without `-c core.hooksPath=/dev/null` are all reported:

```yaml
# .github/workflows/ci.yml
name: Release
on: push
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - run: gh api -f x=1 repos/my-org/my-repo/labels
      - run: gh api "repos/my-org/my-repo/labels?per_page=100"
      - run: 'git commit -m "chore: update"'
```

## Fix

Add the required flags directly to each command and move query parameters from
the positional endpoint into structured `-f` / `-F` fields with `--method GET`:

```yaml
# .github/workflows/ci.yml
name: Release
on: push
jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - run: gh api --method GET -f x=1 repos/my-org/my-repo/labels
      - run: gh api --method GET -F per_page=100 repos/my-org/my-repo/labels
      - run: 'git -c core.hooksPath=/dev/null commit -m "chore: update"'
```

## Suppression

Use a YAML comment (or shell comment in files selected by `shellFiles`) on or
above the reported line, or at the top of the file:

- `# no-mistakes-disable-next-line github-actions-run-command-policy`
- `# no-mistakes-disable-line github-actions-run-command-policy`
- `# no-mistakes-disable-file github-actions-run-command-policy`

Directives apply to policy findings and unanalyzable diagnostics in readable files.

```yaml
# .github/workflows/ci.yml
steps:
  # no-mistakes-disable-next-line github-actions-run-command-policy -- reviewed fixture
  - run: gh api -f x=1 repos/my-org/my-repo/labels
  - run: 'git commit -m "chore: allow"' # no-mistakes-disable-line github-actions-run-command-policy
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` records a warning and runs at
  `max`.

### Unanalyzable inputs

- A selected workflow or action file that cannot be read produces a diagnostic
  at line 1. Invalid YAML produces a diagnostic at the parser's error location
  when available.
- Nested command substitutions and literal shell scripts share a four-level limit.
  When the next body would exceed the limit, the rule reports at that
  substitution or script operand. Shallower commands and siblings continue to
  be checked.
- Dynamic command names (`$CMD ...`, `"$CMD" ...`, `${{ inputs.cmd }} ...`,
  `$DIR/$TOOL ...`, or a variable-prefixed path whose literal final segment
  matches a configured policy `command[0]`, a known shell such as `sh`/`bash`/
  `zsh`/`dash`/`ksh`/`fish`/`pwsh`, or a wrapper command such as `env`/`sudo`/
  `xargs`/`nohup`/`time`/`exec`/`command`),
  `eval`, dynamic `bash -c` / `sh -c` scripts, dynamic subcommands on a
  configured base command, unquoted word-splitting expansions before `--` on a matching
  command whose required arguments are absent, dynamic values on required flags
  (such as `--method "$METHOD"`), dynamic arguments that could match an
  `unlessAnyArgs` exemption when a policy would otherwise fail (such as
  `gh api -f x=1 "$ENDPOINT"` with `unlessAnyArgs: [graphql]`), a dynamic
  word that could expand to a configured flag (including an option-shaped word,
  and only when a requirement or exemption could still fail), to an inline `flag=value` or
  attached short form on a when or unless pair gate, or to a `valueFlags` entry
  that would consume the next exemption or flag, a dynamic word that could expand to
  `--` before a later required flag, a configured token whose text equals an
  expansion's source, a command substitution whose `)` sits
  inside quotes, an escaped closing `)` or a nested `$(...)` whose quotes would
  otherwise end the substitution, and
  unterminated quotes, expansions, or here-documents abstain from policy failure
  and emit a diagnostic governed by the effective unanalyzable policy.
- Variable-prefixed script paths whose final path segment is a literal matching
  no configured policy `command[0]`, known shell, or wrapper command (for
  example `$GITHUB_ACTION_PATH/build-images.sh`) abstain silently with no
  advisory.

## Related rules

- [`gh-api-argument-safety`](../eslint-rules/gh-api-argument-safety.md) checks
  TypeScript and JavaScript `gh api` invocations in ESLint and Oxlint.
- [`github-actions-action-input-policy`](github-actions-action-input-policy.md)
  enforces required and banned `with:` inputs on configured `uses:` actions.
- [`github-actions-download-integrity`](github-actions-download-integrity.md)
  shares the same shell script parser for download verification.
- [`github-actions-script-expressions`](github-actions-script-expressions.md)
  rejects untrusted `${{ }}` interpolation in `run` scripts.

## Invalid UTF-8

Invalid UTF-8 bytes in workflow YAML, action YAML, or selected `shellFiles`
scripts are decoded lossily so definite command-policy violations in the same
file are still reported.
