# `required-doc-section`

Requires a Markdown heading in files matched by a glob.

```yaml
rules:
  - rule: required-doc-section
    scope: repository
    options:
      glob: "agents/*/README.md"
      requiredHeading: "## Perf"
```

Counterexample: a local README missing the required section.

Fix: add the heading and relevant content.

## Why and when

Use this rule when a class of local Markdown files must carry a review or
operational section, such as performance notes or ownership guidance.

## What it catches/requires

Every file matching `glob` must contain the exact `requiredHeading` Markdown
heading. Content below the heading is not inferred or validated by this rule.

## Options and defaults

`glob` and `requiredHeading` are required options; there are no defaults because
silently choosing a document set or heading would create a false policy.

## Valid example

```md
## Perf

This agent keeps its query under 100 ms.
```

## Counterexample

```md
# Perf
```

The configured heading is `## Perf`, so the level-one heading does not satisfy
the contract.

## Fix

Add the exact configured heading to each matched file or correct the glob and
heading together when the policy changed.

## Suppression

Narrow the rule's glob for files that intentionally have a different document
shape. Use `no-mistakes-disable-file required-doc-section` only for generated
or externally owned Markdown.

## Related rules

[`required-local-docs`](required-local-docs.md) ensures a document exists beside
code; [`doc-consistency`](doc-consistency.md) checks broader document policy.
