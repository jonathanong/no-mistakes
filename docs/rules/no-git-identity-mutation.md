# `no-git-identity-mutation`

## Why and when

Use this rule when repository automation must not silently replace the user's
Git identity, which can misattribute commits and leak bot configuration across
jobs.

## What it catches

It catches `git config` commands that write `user.name` or `user.email` in
selected scripts; read-only identity queries remain allowed. Literal global
options before `config` (for example `git -C . config` or
`git --git-dir=.git config`) and single/double quotes around `git` or `config`
are covered too, including a whole quoted `--git-dir=.git` option. Short
options use separate arguments, such as `-C .` or `-c color.ui=false`.
Global option arguments may be quoted shell words, including variables inside
double quotes; their contents are never evaluated. Unquoted command
substitutions and dynamically constructed executable/subcommand words are not
interpreted.

## Options

The only rule-local options are `excludePaths` and
`conditionallyAllowedWorkflows`; both default to empty lists. `excludePaths`
skips matching files entirely. `conditionallyAllowedWorkflows` permits an
otherwise matching workflow only when its contents satisfy the managed-runner
condition. Generic rule `include`/`exclude` filters are separate from these
rule-local exceptions.

## Valid example

`git config user.email` without a value reads identity and passes; configuring
identity outside the repository script also passes.

```sh
git status
```


## Counterexample

```sh
git config user.email ci@example.com
```

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` (on the rule, or as a repository or CLI default) is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Following [recall over precision](../../README.md#how-analysis-works), nothing this rule cannot prove is treated as clean, and it has no such inputs.

- This rule has no unanalyzable inputs: it judges literal identity-changing commands in scripts and workflows. Nothing is routed through `options.unanalyzable`.

## Suppression

Use a line directive for an exceptional command or a file directive only for a
dedicated setup script; prefer moving the mutation out.

## Related rules

[`shellcheck-runner`](shellcheck-runner.md) checks shell correctness but not
repository identity policy.

Bans scripts that mutate git user identity.

```yaml
rules:
  - rule: no-git-identity-mutation
    scope: repository
```

Counterexample: `git config user.email bot@example.com` in setup scripts.

Fix: read git identity when needed, but configure identity outside repository
scripts.
