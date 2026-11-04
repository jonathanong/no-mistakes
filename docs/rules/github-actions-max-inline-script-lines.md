# `github-actions-max-inline-script-lines`

## Why and when

Long inline scripts hide program logic in YAML, where linting and testing are harder.
Enable this rule to keep GitHub Actions steps small and move longer programs into files.

## What it catches

Checks `run` bodies and `actions/github-script` `with.script` bodies in workflows directly
under `.github/workflows/` and composite actions under `.github/actions/**/action.yml`
or `action.yaml`. Other action inputs and non-composite action definitions are ignored.

Counts non-blank lines in the YAML script body, including comment-only script
lines. A physical line ending in `\` joins the next line, so a continued command
counts once. A space after the backslash prevents continuation. Blank lines do not
count; a blank continuation target ends that continuation. Heredoc delimiters,
heredoc contents, and `${{ }}` expressions count as ordinary body lines. Literal (`|`)
and folded (`>`) block scalars use their original body lines before YAML folding,
with the same continuation joining. The script key, YAML quote delimiters, and blank
lines do not count. YAML comments indented less than a block scalar's content do not
count; comments at its content indentation remain script lines, including with explicit
indentation indicators and anchors, regardless of the order of the chomping and indentation
modifiers. For example, with a `run:` key indented eight spaces and `run: &script |2-`,
content starts at indentation ten: a YAML comment at indentation nine is outside the script,
while a comment at indentation ten is part of it and counts. Single-line scalar scripts
count as one line. A script with exactly the limit passes. Findings point to the `run:` or
`script:` key, include the count and limit, and explain the extraction fix.

Literal (`|`) and folded (`>`) block scalars used as the step's `uses` value are recognized
as action references after YAML chomping and folding. Thus `actions/github-script@v7` is
recognized whether written as a plain value or a block scalar, and whether `with.script`
appears before or after `uses` in the step mapping. Findings for these steps still point to
the `with.script` key and count its body using the same rules as a `run:` body.

Malformed YAML produces a parse diagnostic. Script bodies whose locations cannot be aligned
(for example aliases) produce a finding asking for ordinary block-style YAML. In a block-style
steps list, an unsupported flow mapping (`- { ... }`) is reported at its declaration line.
Put a `no-mistakes-disable-next-line` directive
immediately above the flow-style step, or use `no-mistakes-disable-line` on the step line,
to suppress that diagnostic. Matched scripts keep their `run:` or `script:` key location,
and other scripts in the same file are still checked at their own source locations.

## Options

`maxLines` is a positive integer; the default is **10**. Standard rule `include`, `exclude`,
and project selection apply before workflow/action discovery. Multi-line `with:` inputs
that carry a command are outside this rule, except `actions/github-script`
`with.script`.

```yaml
rules:
  - rule: github-actions-max-inline-script-lines
    scope: repository
    options:
      maxLines: 3
```

## Valid example

With `maxLines: 3`, this three-line body passes:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: |
          echo one

          echo two
          echo three
```

## Counterexample

With the same limit, this four-line composite-action body fails:

```yaml
# .github/actions/prepare/action.yml
runs:
  using: composite
  steps:
    - shell: bash
      run: |
        echo one
        echo two
        echo three
        echo four
```

## Fix

Move the program to a script file, lint and test that file, and call it from `run:`.
Composite actions can use `run: bash "$GITHUB_ACTION_PATH/scripts/build.sh"`.
For `actions/github-script`, put the JavaScript in a module and require it from the short
`with.script` body, passing the action's `github`, `context`, and `core` values as needed.
There is no automatic fix because extraction must preserve shell, working directory,
permissions, environment, and module context.

Composite-action findings explain that scripts beside the action need
`$GITHUB_ACTION_PATH`: a repo-relative path depends on the caller's checkout.
For workflows triggered by `workflow_run` or `pull_request_target`, findings
require the moved script to come from the trusted ref, never the PR head.

## Suppression

YAML comment directives work on the script key line, immediately above that line, or for
the whole file:

```yaml
# no-mistakes-disable-next-line github-actions-max-inline-script-lines
- run: ./scripts/build.sh
```

Use `# no-mistakes-disable-line github-actions-max-inline-script-lines` on a `run:` or
`script:` key, or `# no-mistakes-disable-file github-actions-max-inline-script-lines` at the
top of the file. A step before checkout still reports; it is not automatically exempt. For a
justified exception, put the YAML directive immediately above `run:`, even when
`name:` appears earlier in the step.

Suppression caveat: a directive on the step's `name:` or `uses:` does
not suppress a finding on a later script key. For unsupported flow-style steps, the
diagnostic is attached to the step declaration, so put the directive immediately above
that declaration instead. For example:

```yaml
# no-mistakes-disable-next-line github-actions-max-inline-script-lines
- { run: 'echo one; echo two; echo three; echo four' }
- { run: 'echo one; echo two; echo three; echo four' } # no-mistakes-disable-line github-actions-max-inline-script-lines
```

## Related rules

- [`github-actions-script-expressions`](github-actions-script-expressions.md) keeps interpolated input out of scripts.
- [`github-actions-step-timeouts`](github-actions-step-timeouts.md) bounds workflow step runtime.
- [`github-actions-composite-step-schema`](github-actions-composite-step-schema.md) validates composite step keys.
