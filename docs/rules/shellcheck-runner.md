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

## Why and when

Use this rule for checked-in shell automation, especially scripts run by CI or
release tooling where quoting and error handling failures are expensive.

## What it catches/requires

Each selected shell file must pass the installed ShellCheck binary. The rule
does not fabricate diagnostics when ShellCheck is unavailable.

## Options and defaults

There are no rule-local options. The runner uses the repository's configured
file universe and the installed `shellcheck` executable.

## Valid example

```sh
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "${1:-default}"
```

## Counterexample

```sh
#!/usr/bin/env bash
echo $1
```

Unquoted expansion and an unset positional argument are ShellCheck findings.

## Fix

Quote expansions, enable deliberate error handling, and address the reported
SC codes rather than suppressing the entire script.

## Suppression

Use a ShellCheck-specific inline directive only when the code is intentional;
use `no-mistakes-disable-file shellcheck-runner` when the script is generated
and cannot be edited.

## Related rules

[`workflow-topology-policy`](workflow-topology-policy.md) checks CI structure;
this rule checks the shell implementation invoked by that workflow.
