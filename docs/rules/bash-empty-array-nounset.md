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
`${name[*]}` expansions. A list with at least one element that cannot vanish, a scalar
initializer, or a direct scalar or indexed write to a known array establishes a
nonempty array. An element cannot vanish when it is a literal or a
double-quoted expansion such as `"$selected"`, because a quoted word is always
one word, even with `nullglob`. An unquoted glob, an unquoted expansion, and
`"$@"`, `"${name[@]}"`, or `"${!prefix@}"` can produce zero words, so a list made only of those
may be empty. `read -a` and `read -ra` targets, and `mapfile` and `readarray`
targets, are tracked as possibly empty arrays because the input can be empty.
`mapfile -O` keeps the elements the array already had. Replacing it with a list that may be empty or using `unset` on the
array or a literal indexed operand removes that guarantee. The rule treats
any element removal as potentially empty because it tracks whether an array
has elements without counting them.

## Counterexample

```bash
# scripts/legacy/run.sh
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

A `read -ra` target is reported when expanded directly.

```bash
# scripts/legacy/read.sh
set -u
IFS=',' read -ra requested <<< "$1"
collect "${requested[@]}"
```

## Valid example

An array with a literal element or a quoted expansion can be expanded directly.

```bash
# scripts/legacy/run.sh
set -u
items=('example')
collect "${items[@]}"
candidates=("$selected" "$root"/Tool*.app)
collect "${candidates[@]}"
```

Scripts embedded in a workflow `run:` block are not scanned. The rule reads
files selected by `options.files`. Move the script to a file and list it there
to check it.

The check is lexical. It uses source order and skips nested substitutions and
here-document bodies, including bodies introduced with ANSI-C quoted
delimiters. It keeps combined output redirects and adjacent file descriptor
prefixes in the same command. It does not evaluate control flow, function
calls, or sourced files.

## Suppression

Use `# no-mistakes-disable-next-line bash-empty-array-nounset -- <reason>`
above the expansion, `# no-mistakes-disable-line bash-empty-array-nounset -- <reason>`
on the same line, or `# no-mistakes-disable-file bash-empty-array-nounset -- <reason>`
for a whole file. Explain the invariant that guarantees the expansion is safe.

## Related rules

[`zsh-readonly-parameter`](zsh-readonly-parameter.md) covers writes to zsh's
exit status. [`shellcheck-runner`](shellcheck-runner.md) integrates ShellCheck
diagnostics with `check`.
