# `github-actions-script-expressions`

## Why and when

Use this rule when a workflow or composite action builds a shell or JavaScript
script. GitHub expands `${{ }}` before the shell or the script parser runs, so
an event payload, ref, actor, input, output, secret, variable, or environment
value inside `run` or `actions/github-script` is code, not data.

Template-injection checks that drop findings below a confidence cut-off miss
the same shapes. This rule reports every expression in those script bodies
unless it matches an allowed constant context. It does not rank confidence.

## What it catches

The rule scans `jobs.*.steps` in `.github/workflows/*.{yml,yaml}` and
`runs.steps` in composite actions (`.github/actions/**/action.{yml,yaml}` whose
`runs.using` is `composite`). Other action kinds, including Docker `args`, are
not script bodies.

It flags `${{ <expr> }}` in:

- a step `run` body
- `with.script` when the step `uses` `actions/github-script` (a trailing slash
  is the same action; `actions/github-script/extra` is not)

Values already in `env` or in `with` (other than that `script` input) are not
flagged. `if`, `defaults.run`, workflow `env`, job `env`, and reusable-workflow
`with` are not script bodies. The fix for everything else is to pass the value
through `env`, where the shell or JavaScript reads it as data.

Invalid YAML is one finding. The rule does not guess expressions in a file it
cannot parse. Anchors, aliases, and flow-style steps that the line scan cannot
align with the parsed script are one unanalyzable finding for the file. Rewrite
them as ordinary block-style `run` or `with.script` bodies. An expression split
across lines is unanalyzable for the same reason.

## Options

`constantContexts` is a list of regular expressions. A pattern allows an
expression only when the trimmed expression is a full match, not a substring.
Matching is case-sensitive.

When the rule is enabled and `constantContexts` is omitted, the default is:

```text
^github\.(repository|repository_owner|server_url|run_id|run_number|run_attempt|sha|event_name|workspace)$
^runner\.(os|arch|temp)$
^job\.services\.[A-Za-z0-9_-]+\.ports\[(?:'[0-9]+'|"[0-9]+")\]$
```

The service-port pattern allows `job.services.<id>.ports['6379']` and
`job.services.<id>.ports["5432"]`. Unquoted `ports[6379]` does not match.
Add a pattern when a workflow uses that form on purpose.

An explicit list replaces the default. `constantContexts: []` allows nothing,
so every script expression is flagged.

```yaml
rules:
  - rule: github-actions-script-expressions
    scope: repository
```

An explicit `constantContexts` list replaces the default:

```yaml
rules:
  - rule: github-actions-script-expressions
    scope: repository
    options:
      constantContexts:
        - '^github\.ref$'
```

The rule is opt-in. Listing it is what turns it on. Product defaults do not
enable it.

## Valid example

Constant contexts may stay inline. Untrusted values go through `env`.

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - env:
          TITLE: ${{ github.event.pull_request.title }}
        run: echo "$TITLE"
      - run: echo "${{ github.sha }}"
      - uses: actions/github-script@v7
        env:
          ACTOR: ${{ github.actor }}
        with:
          script: return process.env.ACTOR
```

A composite action passes `inputs` the same way:

```yaml
runs:
  using: composite
  steps:
    - shell: bash
      env:
        NAME: ${{ inputs.name }}
      run: echo "$NAME"
```

## Counterexample

This `run` and this `actions/github-script` body expand attacker-controlled
text before the script parser runs:

```yaml
- run: echo "${{ github.event.head_commit.message }}"
- uses: actions/github-script@v7
  with:
    script: |
      return "${{ github.event.issue.title }}"
```

## Fix

Pass the value through `env` and read it as data:

```yaml
- env:
    MESSAGE: ${{ github.event.head_commit.message }}
  run: echo "$MESSAGE"
- uses: actions/github-script@v7
  env:
    TITLE: ${{ github.event.issue.title }}
  with:
    script: return process.env.TITLE
```

For a shell script the same shape is `env: VALUE: ${{ <expr> }}`, then
`run: echo "$VALUE"`.

## Suppression

Use a YAML comment. `no-mistakes-disable-next-line github-actions-script-expressions`
suppresses the next line. `no-mistakes-disable-line github-actions-script-expressions`
suppresses the same line. `no-mistakes-disable-file github-actions-script-expressions`
in the leading comment block suppresses the file.

```yaml
# no-mistakes-disable-next-line github-actions-script-expressions
- run: echo "${{ github.ref }}"
- run: echo "${{ github.actor }}" # no-mistakes-disable-line github-actions-script-expressions
```

## Related rules

[`github-actions-pinned-hash`](github-actions-pinned-hash.md) pins action code.
[`github-actions-job-timeouts`](github-actions-job-timeouts.md) and
[`github-actions-step-timeouts`](github-actions-step-timeouts.md) bound runtime.
[`github-actions-download-integrity`](github-actions-download-integrity.md)
flags unverified `curl` and `wget` downloads. This rule only judges expressions
inside script bodies.
