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
