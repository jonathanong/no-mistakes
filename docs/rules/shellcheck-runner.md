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

`shellFiles` adds explicit existing repository-relative script paths; it
defaults to `[]`. `shebangDirs` adds non-`.sh` files below the listed
repository-relative directories when their first line is a supported Bash or
sh shebang; it also defaults to `[]`. Selected `.sh` files are always checked.

`skillsLockfile` optionally names a repository-relative skills lockfile for the
rule's path-bearing configuration. It defaults to unset and does not add files
to the ShellCheck candidate set; candidates still come from `.sh`, supported
shebangs, and `shellFiles`.

`shellcheck.severity` sets ShellCheck's minimum severity and defaults to
`warning`; accepted values are `error`, `warning`, `info`, and `style`.
The runner still requires the installed `shellcheck` executable.

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
