# `github-actions-no-poll-wait-job`

## Why and when

A job whose steps are a `sleep` loop polling another job or workflow burns
runner minutes while it waits. Workflows should react to events instead:
`needs` for work in the same workflow, `workflow_run` for another workflow,
and `merge_group` for merge-queue checks.

The rule is opt-in. Listing it is what turns it on. It is not part of a
recommended or strict preset. It has no built-in command names: it reports only
the poll commands that `options.pollCommands` lists, and an empty or missing
list reports nothing.

## What it catches

The rule reads workflow files directly under `.github/workflows/*.{yml,yaml}`
and composite `action.yml` / `action.yaml` files under `.github/actions/`. It
reads `run:` strings on workflow job `steps` and on composite `runs.steps`.

It reports a `while`, `until`, or `for` loop in a `run` script when both hold:

- a command in the loop is `sleep`, and
- a command in the loop condition (`while` and `until`) or body runs a
  configured poll command. A `for` loop is judged by its body only. The poll
  command counts at a command position, including inside `$(...)`, backticks,
  and `[ "$(...)" = ... ]` tests. It does not count as an argument, so
  `echo gh run view` is not a poll.

Each reported loop is one finding on the line of the `while`, `until`, or
`for` keyword. A nested loop is judged on its own, including when it starts
on the same line as the outer loop's `do`. The rule skips here-document bodies
in opener order, including when a single line opens several. If a delimiter is
missing, the rule still reports completed loops before the unfinished body.

These forms are silent:

- `sleep` outside a loop.
- A loop with no configured poll command, or a poll command with no `sleep`.
- A stray `done` and loops with no matching `done`. An
  unfinished here-document can hide a closing `done`, so the rule only judges
  completed loops in that case.
- A `for` loop, when `options.ignoreForLoops` is `true`. A `for` loop runs a
  fixed number of times, so a bounded retry is not a wait.

## Options

`options.pollCommands` is a list of command prefixes such as `gh run view`.
Each entry must name at least one word. The default is an empty list.
`options.ignoreForLoops` defaults to `false`. Unknown options are rejected.

```yaml
rules:
  - rule: github-actions-no-poll-wait-job
    scope: repository
    options:
      pollCommands:
        - gh run view
```

## Valid example

Depend on the work with `needs` instead of polling for it:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: make build
  report:
    runs-on: ubuntu-latest
    needs: build
    steps:
      - run: gh run view 123 --json status
```

## Counterexample

This job loops with `sleep` until another run finishes:

```yaml
jobs:
  wait:
    runs-on: ubuntu-latest
    steps:
      - run: until gh run view 123 --json status; do sleep 30; done
```

## Fix

Make the waiting job a dependent job with `needs`, or trigger it from the
event that signals completion (`workflow_run` for another workflow,
`merge_group` for the merge queue). When a bounded retry is intended, write a
`for` loop and set `options.ignoreForLoops` to `true`.

## Analysis

### Effort

- Runs at `max` only. Configuring `effort: low` is not an error: `check` records the warning `requested effort low; effective effort max because this rule has no low implementation` and runs the rule at `max`.

### Unanalyzable inputs

When a selected file has invalid YAML, the rule reports the parser location.
A selected file that cannot be read is reported at line 1.
The effective `unanalyzable` policy controls these diagnostics. Use `report`
(the default) for a finding, `advise` for an advisory, or `ignore` to omit it.
Restore read access or fix the YAML syntax so the rule can verify the file. Valid sibling files are
still checked.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-no-poll-wait-job`
suppresses the next line. `no-mistakes-disable-line github-actions-no-poll-wait-job`
suppresses the same line. `no-mistakes-disable-file github-actions-no-poll-wait-job`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-no-poll-wait-job
- run: until gh run view 123; do sleep 30; done
- run: until gh run view 123; do sleep 30; done # no-mistakes-disable-line github-actions-no-poll-wait-job
```

## Related rules

[`github-actions-job-timeouts`](github-actions-job-timeouts.md) bounds how long
a job may run. [`github-actions-step-timeouts`](github-actions-step-timeouts.md)
bounds a step. This rule flags the polling loop that makes a job wait.

## Invalid UTF-8

Invalid UTF-8 is decoded lossily and scanned, so a violation next to a stray byte is still reported. A file that cannot be read for any other reason is skipped and produces no finding.
