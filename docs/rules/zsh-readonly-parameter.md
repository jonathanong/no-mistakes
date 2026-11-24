# `zsh-readonly-parameter`

## Why and when

Zsh reserves `status` as its read-only exit status. Code that uses the name as a
variable can fail with a read-only variable error in zsh.
Enable this rule for scripts run or sourced with zsh.

```yaml
rules:
  - rule: zsh-readonly-parameter
    scope: repository
    options:
      files: ["scripts/startup"]
```

## Options

The rule selects files with a `.zsh` extension or a shebang that names zsh
directly or through `env`. The rule recognizes `-S zsh -f`, `-S/bin/zsh -f`,
`--split-string zsh -f`, and `--split-string=zsh -f`, with support for
`-i` (`--ignore-environment`) and the `--` option separator. Quote individual
arguments with spaces. Quoting the entire `'zsh -f'` names a single executable.
Only files that look like shell scripts are read to look for a shebang: files
with no extension, a `.sh` or `.zsh` extension, or an executable bit. A zsh file
with neither a `.zsh` extension nor a zsh shebang, or with another extension and
no executable bit, needs an `options.files` glob. Use `options.files` to add
repository-relative globs for sourced files and other shebang forms. The option defaults to an empty list. The standard rule
and project `include` and `exclude` filters further restrict the scope. The
rule works without ShellCheck.

## What it catches

The finding points to the line that assigns or declares `status`. The rule
recognizes direct assignments and parameter declarations with `local`,
`typeset`, or `declare`. Function declarations and queries in `typeset -f`
mode use a separate namespace.
Only `status` is checked. Other read-only zsh parameters stay out until each is
verified against the zsh parameter manual (`zshparam`). Scripts embedded in a
workflow `run:` block are not scanned, whatever the step's `shell:`. Move the
script to a file and list it in `options.files`.
It checks the name `status` from the [zsh parameter manual](https://zsh.sourceforge.io/Doc/Release/Parameters.html#Parameters-Set-By-The-Shell).
It uses lexical command positions and skips nested substitutions and
here-document bodies, including bodies introduced with ANSI-C quoted
delimiters. It keeps combined output redirects and adjacent file descriptor
prefixes in the same command.

## Counterexample

```zsh
report() {
  local status=$?
  status=$(collect)
}
```

## Fix

Rename the variable and update its references.

```zsh
report() {
  local exit_status=$?
  exit_status=$(collect)
}
```

## Valid example

Reading `status` is valid.

```zsh
print -r -- "$status"
```

A local declaration with `-h` hides the special parameter through the end of
its brace-delimited function body. An ordinary brace group shares that scope.
At file scope or with `-g`, the `-h` attribute can affect later localization.
Assignments to the active special parameter still fail.

## Suppression

Use `# no-mistakes-disable-next-line zsh-readonly-parameter -- <reason>` above
the offending line, `# no-mistakes-disable-line zsh-readonly-parameter -- <reason>`
on the same line, or `# no-mistakes-disable-file zsh-readonly-parameter -- <reason>`
for a whole file. Explain why the assignment or declaration is safe.

## Related rules

[`bash-empty-array-nounset`](bash-empty-array-nounset.md) covers empty array
expansions in scripts targeting older Bash releases.
[`shellcheck-runner`](shellcheck-runner.md) integrates ShellCheck diagnostics
with `check`.
