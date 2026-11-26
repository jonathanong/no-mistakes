# `github-actions-pr-diff-range`

## Why and when

A pull-request change set must come from a merge-base diff: a three-dot diff
with enough history fetched, or `git diff HEAD^1 HEAD` on the merge commit. A
two-dot or two-argument `git diff <base.sha> <head.sha>` also lists
base-branch changes made after the pull request branched, so path filters and
changed-file checks run on the wrong set.

Pairing the event's base SHA with `github.sha` is a second defect. The two
values are computed at different times, and on other events (`push`,
`merge_group`) they mean different things. Merge-queue runs must resolve their
own range from `github.event.merge_group.base_sha` and `head_sha`.

The rule is opt-in. Listing it is what turns it on. It is not part of a
recommended or strict preset.

## What it catches

The rule reads `run:` strings on job `steps` of workflow files directly under
`.github/workflows/*.{yml,yaml}`. Composite actions do not declare triggers,
so they are not judged. The workflow's `on:` events choose what is judged.

In a workflow triggered by `pull_request` or `pull_request_target`, it reports
a `git diff` whose range is:

- a two-dot (`base..head`) or two-argument (`base head`) range whose ends are
  `github.event.pull_request.base.sha` and
  `github.event.pull_request.head.sha`, in either order, or
- a range of any form (`..`, `...`, or two arguments) that pairs
  `github.event.pull_request.base.sha`, or `github.base_ref` (also written
  `origin/${{ github.base_ref }}`), with `github.sha`. This pairing is judged
  only when the workflow can also run on `push` or `merge_group`, where
  the same expression can refer to a different revision. The rule skips this
  pairing when neither event is declared. For example, a `pull_request`-only
  workflow can use
  `git diff origin/${{ github.base_ref }}...${{ github.sha }}` to diff against
  the PR merge commit. [Event gating](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_workflow_files/pr_diff_range/runs.rs#L128-L137).

In a workflow triggered by `merge_group`, it reports a step that reads
`github.event.pull_request.*`, which is empty there. A read through an `env`
variable matches the full name in `$NAME` or `${NAME}` and is reported on the
effective `env` entry. Step entries take precedence over job entries, which
take precedence over workflow entries. A step or job whose `if` mentions
`github.event_name` is treated as guarded and is not reported. A single
expression may join the corresponding pull-request and merge-group base SHAs,
or the corresponding head SHAs, with `||`. The fallback covers the
merge-group read. The rule still checks the diff form on pull-request runs.

The rule also recognizes shell `if` / `elif` / `else` / `fi` event-name guards
comparing a quoted event-name variable (or unquoted inside `[[ ... ]]`) to a
literal event name such as `pull_request`, `pull_request_target`, or
`merge_group`. The variable must either be the runner's `GITHUB_EVENT_NAME`
(when not overridden in `env`) or have an effective `env` value of
`${{ github.event_name }}`, and no preceding statement (including `if` / `elif`
conditions and `!`-negated commands) may reassign it or run any command outside
a small non-mutating allowlist (`:`, `true`, `false`, `echo`, `set`, `shopt`,
`[` and `test` without subscripted `-v` operands, `[[` without arithmetic or
name-reference operators, `exit`, `return`, `cat`, `git`, literal `printf`
without `-v`, and simple static-target `export` / `readonly` / `local` /
`declare` / `typeset` / `unset` without compound array assignments or
name-reference (`-n`), function (`-f`), or integer (`-i`) flags). Any unlisted
command, dynamic command or target name, compound array assignment, arithmetic
command (`(( ... ))`), command or arithmetic substitution (`$(...)`, `$((...))`,
`$[...]`, or backticks, including inside redirections and unquoted
here-document bodies), or non-simple parameter expansion (`${X:-...}`,
`${X:=...}`) is treated as mutating every variable. Quoted here-document bodies
(`<<'EOF'`, `<<"EOF"`, `<<\EOF`) do not expand in shell and are skipped when
scanning for expansions. The rule skips
environment reads in branches that `merge_group` cannot reach — including nested
`if` blocks such as regex checks inside a guarded branch — and checks reads in
reachable branches and outside the `if`. Compound tests and other unmodeled
control flow (`case`, loops, grouped commands, functions, `time`, `coproc`)
abstain from excluding branches so any `github.event.pull_request.*` read is
still reported.

```yaml
- env:
    EVENT_NAME: ${{ github.event_name }}
    PR_BASE_SHA: ${{ github.event.pull_request.base.sha }}
    MERGE_GROUP_BASE_SHA: ${{ github.event.merge_group.base_sha }}
  run: |
    if [ "$EVENT_NAME" = "pull_request" ]; then
      base="$PR_BASE_SHA"
    else
      base="$MERGE_GROUP_BASE_SHA"
    fi
    echo "$base"
```

The equivalent base-SHA fallback fits in a single environment entry.

```yaml
BASE_SHA: ${{ github.event.pull_request.base.sha || github.event.merge_group.base_sha }}
```

A range end is recognized when it is the expression itself, optionally behind
`origin/` or `refs/remotes/origin/`, or a `$NAME` / `${NAME}` variable that
a workflow, job, or step `env` assigns to that expression. This includes
the supported pull-request and merge-group fallbacks above. [Revision extraction](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/github_actions_workflow_files/pr_diff_range/exprs.rs#L32-L90). Anything not recognized as a diff range is
silent: another command, a `git diff` flag that takes a separate value
(`-U 3`, `--output file`), more than two revisions, a revision the rule does
not know, here-document bodies, and invalid YAML. The scanner skips
here-document bodies in opener order, including when several open on the same
line. A missing delimiter leaves earlier diff commands available for analysis.
The fetch depth behind a three-dot diff is not judged.

## Options

This rule has no options, so there is no configurable default. Listing the rule
is what turns it on.

```yaml
rules:
  - rule: github-actions-pr-diff-range
    scope: repository
```

## Valid example

Take a three-dot diff of the base and head SHAs after fetching enough history:

```yaml
on: pull_request
jobs:
  changes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - run: git diff --name-only ${{ github.event.pull_request.base.sha }}...${{ github.event.pull_request.head.sha }}
```

## Counterexample

A two-argument diff between the base and head SHAs lists base-branch changes
made after the branch point:

```yaml
on: pull_request
jobs:
  changes:
    runs-on: ubuntu-latest
    steps:
      - run: git diff --name-only ${{ github.event.pull_request.base.sha }} ${{ github.event.pull_request.head.sha }}
```

## Fix

Use a three-dot diff with sufficient `fetch-depth`, or diff the merge commit
with `git diff HEAD^1 HEAD`. For a merge-queue run, use
`github.event.merge_group.base_sha` and `github.event.merge_group.head_sha`.
In a workflow that also runs on `push` or `merge_group`, do not pair
`github.event.pull_request.base.sha` or `github.base_ref` with `github.sha`.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

Invalid YAML and commands the parser does not read produce no finding. They
are not sent through `options.unanalyzable`.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-pr-diff-range`
suppresses the next line. `no-mistakes-disable-line github-actions-pr-diff-range`
suppresses the same line. `no-mistakes-disable-file github-actions-pr-diff-range`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-pr-diff-range
- run: git diff --name-only ${{ github.event.pull_request.base.sha }} ${{ github.event.pull_request.head.sha }}
- run: git diff --name-only ${{ github.event.pull_request.base.sha }} ${{ github.event.pull_request.head.sha }} # no-mistakes-disable-line github-actions-pr-diff-range
```

## Related rules

[`github-actions-script-expressions`](github-actions-script-expressions.md)
rejects untrusted expressions inside `run` scripts.
[`github-actions-checkout-credentials`](github-actions-checkout-credentials.md)
judges how a checkout is configured. This rule judges the revision range of a
`git diff` in pull-request and merge-queue workflows.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
