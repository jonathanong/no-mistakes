# `bash-empty-array-nounset`

## Why and when

Bash before 4.4 treats an empty array as unbound under `set -u` (nounset).
Expanding the array can abort a script before it runs the intended command.
Enable this rule for scripts that must run on those Bash versions.

```yaml
rules:
  - rule: bash-empty-array-nounset
    scope: repository
    options:
      files: ["scripts/legacy/**/*.sh"]
```

## Options

| Option | Meaning |
| --- | --- |
| `files` | Repository-relative path globs for scripts that must run on Bash before 4.4. Defaults to an empty list, which selects zero scripts. |

The standard rule and project `include` and `exclude` filters further restrict
the scope. The rule works without ShellCheck.

## What it catches

The finding points to the expansion line. The check tracks direct array
initializers and appends and `declare -a` and `local -a` declarations in the
same file. It follows explicit `set` options in source order, including
`set -o nounset` and `set +u`. It checks quoted `${name[@]}` and
`${name[*]}` expansions. A list with a literal element stays safe until it
is replaced by a list that may be empty or unset.

## Counterexample

```bash
set -euo pipefail
items=()
collect "${items[@]}"
```

## Fix

Use the presence guard to preserve argument boundaries and pass zero
arguments for an empty array.

```bash
collect ${items[@]+"${items[@]}"}
```

Keep the `*` selector if the command expects one joined argument for a
nonempty array.

```bash
collect ${items[*]+"${items[*]}"}
```

## Valid example

An array with a literal element can be expanded directly.

```bash
set -u
items=('example')
collect "${items[@]}"
```

The check is lexical. It uses source order and skips nested substitutions and
here-document bodies. It does not evaluate control flow, function calls, or
sourced files.

## Suppression

Use `# no-mistakes-disable-next-line bash-empty-array-nounset -- <reason>`
above the expansion, `# no-mistakes-disable-line bash-empty-array-nounset -- <reason>`
on the same line, or `# no-mistakes-disable-file bash-empty-array-nounset -- <reason>`
for a whole file. Explain the invariant that guarantees the expansion is safe.

## Related rules

[`zsh-readonly-parameter`](zsh-readonly-parameter.md) covers writes to zsh's
exit status. [`shellcheck-runner`](shellcheck-runner.md) integrates ShellCheck
diagnostics with `check`.
