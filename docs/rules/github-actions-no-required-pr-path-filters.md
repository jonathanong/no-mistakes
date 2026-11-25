# `github-actions-no-required-pr-path-filters`

## Why and when

A workflow that supplies a required pull-request check must start for every
pull request in its required scope. If GitHub skips the workflow because of a
path filter, the required check can stay pending and block the pull request.

## Options

```yaml
version: 2
rules:
  - rule: github-actions-no-required-pr-path-filters
    scope: repository
    options:
      workflows:
        - .github/workflows/ci.yml
```

`workflows` lists exact repository-relative `.yml` and `.yaml` workflow paths.
The default empty list selects no files. Rule and project scopes can narrow
the list further. Use the workflows that publish required checks in your
branch protection or rulesets.

## What it catches

In a selected workflow, the rule reports each `paths` or `paths-ignore` key
under `on.pull_request` or `on.pull_request_target`.

## Valid example

The PR trigger starts the workflow for any changed path. The push filter
applies only to push events.

```yaml
on:
  pull_request:
  push:
    paths: ['src/**']
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - run: ./scripts/verify
```

## Counterexample

A pull request that changes only documentation can skip this workflow.

```yaml
on:
  pull_request:
    paths: ['src/**']
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - run: ./scripts/verify
```

## Fix

Remove the PR path filter. If work needs to be conditional, detect changes
inside the workflow and keep an always-reporting gate that handles the
results from the selected jobs.

The rule inspects the workflow declaration. It does not query GitHub or
verify job conditions, activity types, branch filters, or the required
status check's settings. A selected workflow that cannot be read or parsed
follows the shared `unanalyzable` policy.

## Suppression

Use `no-mistakes-disable-line github-actions-no-required-pr-path-filters` on
a reported key, `no-mistakes-disable-next-line` on the preceding line, or
`no-mistakes-disable-file` for the selected file.

Run `no-mistakes check --format json` or use async `check()` and
`analyzeProject()` with the same config.

## Related rules

[`workflow-topology-policy`](workflow-topology-policy.md)
checks declared jobs and their dependency edges in configured workflows.

[GitHub workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)
