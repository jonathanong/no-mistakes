# `shellcheck-runner`

Runs ShellCheck on shell files and configured shell scripts.

```yaml
rules:
  - rule: shellcheck-runner
    scope: repository
    options:
      shellcheck:
        severity: info
      unanalyzable: report
```

## Why and when

Use this rule for checked-in shell automation, especially scripts run by CI or
release tooling where quoting and error handling failures are expensive.

## What it catches/requires

Each selected shell file must pass the installed ShellCheck binary. A ShellCheck
finding is a proven script issue. If at least one script is selected but the
`shellcheck` executable is missing, the rule cannot analyze those scripts and
routes that uncertainty through the shared `unanalyzable` policy. It does not
claim the scripts are clean.

The policy defaults to the repository `unanalyzable` setting, which defaults to
`report`; `options.unanalyzable` can override it for this rule. `report` emits a
failing unanalyzable diagnostic, `advise` emits a non-failing advisory, and
`ignore` skips the missing-tool uncertainty. Install ShellCheck in the
environment that runs `no-mistakes` to analyze the selected scripts. This rule
has no low-effort variant; a low-effort invocation runs it at `max`.

## Options and defaults

`shellFiles` adds explicit existing repository-relative script paths; it
defaults to `[]`. `shebangDirs` adds non-`.sh` files below the listed
repository-relative directories when their first line is a supported Bash or
sh shebang; it also defaults to `[]`. Selected `.sh` files are always checked.

`trackedOnly` defaults to `false`. When `true`, the rule selects only files
present in the Git index and working tree, including modified or staged files.
It filters `.sh` files, supported shebang candidates, and explicit `shellFiles`
against the same tracked view. Outside a Git checkout, this option uses the
ignore-aware visible file set because no Git index exists. With the default
`false`, visible untracked files and explicit paths keep their existing behavior.
The tracked view comes from the request's shared discovery snapshot, so the
option does not start another file scan.

`skillsLockfile` optionally names a repository-relative skills lockfile for the
rule's path-bearing configuration. It defaults to unset and does not add files
to the ShellCheck candidate set; candidates still come from `.sh`, supported
shebangs, and `shellFiles`.

`shellcheck.severity` sets ShellCheck's minimum severity and defaults to
`warning`; accepted values are `error`, `warning`, `info`, and `style`.
`options.unanalyzable` accepts `report`, `advise`, or `ignore`. It inherits the
repository `unanalyzable` policy and defaults to `report` when neither scope
sets a value. It applies when selected scripts cannot be checked because the
ShellCheck executable is missing. This is independent of `shellcheck.severity`.

## Example

A selected script with no ShellCheck findings:

```sh
#!/usr/bin/env bash
# scripts/run.sh
set -euo pipefail
printf '%s\n' "${1:-default}"
```

## Counterexample

```sh
#!/usr/bin/env bash
# scripts/run.sh
echo $1
```

ShellCheck reports the unquoted positional argument as SC2086. If the executable
is missing, this selected input follows `unanalyzable` instead of being treated
as passing.

```sh
#!/usr/bin/env bash
# scripts/run.sh
run() {
  local name=$(basename "$1")
  printf '%s\n' "$name"
}
run "${1:-default}"
```

Combining `local` with a command substitution can hide the command's failure
status from `set -e`; ShellCheck reports this as SC2155 when installed.

## Fix

Install ShellCheck in the analysis environment so selected scripts can be
checked. Then quote expansions, enable deliberate error handling, and address
reported SC codes rather than suppressing the entire script. For an intentional
temporary exception, set `options.unanalyzable` to `advise` or `ignore` with an
explicit plan to restore ShellCheck coverage.

## Suppression

Use a ShellCheck-specific inline directive only when the code is intentional;
use `no-mistakes-disable-file shellcheck-runner` when the script is generated
and cannot be edited.

## Related rules

[`workflow-topology-policy`](workflow-topology-policy.md) checks CI structure;
this rule checks the shell implementation invoked by that workflow.
