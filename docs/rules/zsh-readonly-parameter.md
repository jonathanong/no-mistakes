# `zsh-readonly-parameter`

Zsh reserves `status` as its read-only exit status. Code that uses the name as a
variable can fail with a read-only variable error in zsh.

```yaml
rules:
  - rule: zsh-readonly-parameter
    scope: repository
    options:
      files: ["scripts/startup"]
```

## Options

The rule selects files with a `.zsh` extension or a zsh shebang. Use
`options.files` to add repository-relative globs for sourced files that lack
either marker. The standard rule and project `include` and `exclude` filters
further restrict the scope. The rule works without ShellCheck.

## Example

```zsh
report() {
  local status=$?
  status=$(collect)
}
```

The finding points to the line that assigns or declares `status`. The rule
recognizes direct assignment and `local`, `typeset`, or `declare`.
It checks the name `status` from the [zsh parameter manual](https://zsh.sourceforge.io/Doc/Release/Parameters.html#Parameters-Set-By-The-Shell).
It uses lexical command positions and skips nested substitutions and
here-document bodies.

## Fix

Rename the variable and update its references.

```zsh
report() {
  local exit_status=$?
  exit_status=$(collect)
}
```

## Counterexample

Reading `status` is valid.

```zsh
print -r -- "$status"
```

A local declaration with `-h` hides the special parameter and allows
assignments in the same function.

## Suppression

Use `# no-mistakes-disable-next-line zsh-readonly-parameter -- <reason>` above
the offending line, `# no-mistakes-disable-line zsh-readonly-parameter -- <reason>`
on the same line, or `# no-mistakes-disable-file zsh-readonly-parameter -- <reason>`
for a whole file. Explain why the assignment or declaration is safe.
