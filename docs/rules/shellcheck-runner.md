# `shellcheck-runner`

Runs ShellCheck on shell files and configured shell scripts.

```yaml
rules:
  - rule: shellcheck-runner
    scope: repository
```

Counterexample: a checked-in `.sh` file with ShellCheck failures.

Fix: address the ShellCheck finding. If `shellcheck` is not installed, the rule
skips silently.
