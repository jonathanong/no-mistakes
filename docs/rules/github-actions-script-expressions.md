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
- `with.script` when the step `uses` `actions/github-script` (owner/repo case
  is ignored; a trailing slash is the same action; `actions/github-script/extra`
  is not)

Values already in `env` or in `with` (other than that `script` input) are not
flagged. `if`, `defaults.run`, workflow `env`, job `env`, and reusable-workflow
`with` are not script bodies. The fix for everything else is to pass the value
through `env`, where the shell or JavaScript reads it as data.

If a selected workflow or composite-action file cannot be read, including
because it contains invalid UTF-8, the rule cannot inspect its scripts. That
input follows `options.unanalyzable`: `report` (the default) emits a failing
diagnostic, `advise` emits non-failing advice, and `ignore` omits the
uncertainty. Restore readable text or correct the file encoding so the rule can
inspect the scripts. The diagnostic is attached to the selected file at line 1.

Invalid YAML remains a definite finding under every unanalyzable policy.
Anchors, aliases, and flow-style steps that the line scan cannot align with the
parsed script remain definite unanalyzable findings under every policy.
Expressions split across lines also remain definite findings for the same
reason. Rewrite these shapes as ordinary block-style `run` or `with.script`
bodies; changing `options.unanalyzable` does not suppress them.

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

Both findings and non-failing advice honor suppression. A source-read
unanalyzable diagnostic is attached at line 1, so suppress it with
`no-mistakes-disable-file github-actions-script-expressions` in the file's
leading comment block or `no-mistakes-disable-line
github-actions-script-expressions` on line 1. A next-line directive before
line 1 cannot suppress it. Definite script findings retain their actual capture
lines and support `disable-line` or `disable-next-line` at those locations.
`no-mistakes-disable-file github-actions-script-expressions` suppresses either
kind for the whole file.

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
