# `doc-consistency`

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
