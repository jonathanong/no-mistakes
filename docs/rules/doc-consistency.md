# `doc-consistency`

## Why and when

Use this rule when a repository needs a declarative documentation contract that
cannot be expressed by link reachability alone.

## What it catches

It requires configured files, headings, and substrings, and rejects configured
stale substrings in selected documentation.

## Options

`requiredFiles`, `requiredSubstrings`, and `bannedSubstrings` default to empty
lists. `requiredHeading` defaults to unset. Each `requiredSubstrings` item names
one `file` and one literal `substring`; leaving every option at its default makes
the rule a no-op.

## Valid example

A matching document containing every required heading and text and none of the
banned text passes.

## Related rules

[`required-doc-section`](required-doc-section.md) is the narrower heading-only
policy; [`markdown-reachability`](markdown-reachability.md) validates discovery.

## Suppression

Use a line directive for a single text finding when it has a line location; use
a file directive only for generated documentation. Prefer narrowing the matching
file or substring policy so future documentation remains checked.

Requires documentation files, headings, substrings, and banned-substring
policies.

```yaml
rules:
  - rule: doc-consistency
    scope: repository
    options:
      requiredFiles: [README.md]
      requiredHeading: "## Install"
```

Counterexample: a README that omits the install section or points to stale docs.

Fix: add the required file/heading/substrings and remove banned text.
