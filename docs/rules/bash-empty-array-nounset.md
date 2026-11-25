<!-- cspell:ignore subshell subshells elif esac -->

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
| `unanalyzable` | How to report unmodeled control-flow, dynamic `unset`, or loop-widened array shapes (`"advise"` by default, or `"report"` / `"ignore"`). |

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
one word, even with `nullglob`. A copy such as `copy=("${items[@]}")` is also
nonempty when `items` is already known to be nonempty. An unquoted glob, an
unquoted expansion, and `"$@"`, `"${name[@]}"`, or `"${!prefix@}"` can produce
zero words, so a list made only of those may be empty. `read -a` and `read -ra` targets, and `mapfile` and `readarray`
targets, are tracked as possibly empty arrays because the input can be empty.
Expanding one of those arrays inside a guarding condition the rule does not
model is unanalyzable; expanding it with no guard stays a definite finding.
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

A length check can reject empty input before expanding the array.

```bash
set -u
IFS=',' read -ra items <<< "$1"
if [[ ${#items[@]} -eq 0 ]]; then
  exit 1
fi
collect "${items[@]}"
```

The check also recognizes `-lt 1`, `[ "${#items[@]}" -eq 0 ]`,
`(( ${#items[@]} == 0 ))`, and `(( ! ${#items[@]} ))`. For a command
list, an empty test can use `&& exit` and a nonempty test can use
`|| exit`. A brace group can log a message before the exit, as in
`(( ${#items[@]} )) || { echo "Expected items" >&2; exit 1; }`.
Use `return` in a function or `continue` or `break` in a loop
to establish the same guarantee on the path that reaches the expansion.
A guard that only prints a message still leaves the empty path reachable.
The guarantee applies to the tested array in the same function or loop.
A later assignment is checked on its own, and `unset` removes the guarantee.

Guards inside a subshell, a command substitution, a pipeline segment, or a
function defined with a parenthesized body apply only within that scope.
For example, if a `case` arm exits from a subshell, the outer shell can still
reach a later array expansion with an empty array. Put the length check in
the outer shell before the expansion.

In a `case` statement with a catch-all `*` arm, arms that unconditionally
leave (`exit`, `return` in a function, `continue` / `break` in a loop, or a
call to an earlier same-file function that unconditionally exits) are excluded
when merging post-`case` array facts. `;&` only advances to the next arm, so
the arm it reaches still decides whether the `case` leaves. An arm that cannot
run, because an earlier pattern already matches every subject and control does
not fall through to it, is not checked for an empty expansion. When every
surviving arm establishes a nonempty array, expanding it after `esac` is
accepted. When an array is explicitly initialized empty before a `case` without
a catch-all `*` arm, or a surviving arm leaves the array possibly empty, the
expansion after `esac` still reports.

Global arrays known to be nonempty before a function definition stay nonempty
inside that function body unless the array is potentially emptied or unset
before the last call to that function (or inside a loop or function). Writes
inside an isolated subshell `( ... )` or `f() ( ... )` and writes to a `local`
variable inside another function do not invalidate a global array. Both
`local -a` and bare `local name` (and `declare` without `-g` inside a function)
keep array assignments local to the function and shadow any global array of the
same name. Appends (`+=`), `mapfile -O`, and top-level `declare -a` without an
assignment never clear a known nonempty array fact. A loop body that empties
and refills an array on every path to `done`, `continue`, or `break` keeps its
nonempty fact across iterations and after `done`, `while` and `until` loops
retain condition writes (such as `while read -ra items`) on loop exit, and
`while :`, `while true`, and `until false` loops merge post-loop state from
their `break` paths. Direct calls to an earlier brace-body function that
populates a non-local array on every returning path establish that nonempty
fact at the call site, and calls to an earlier function whose body
unconditionally exits are treated as leaving paths. A same-file function that
reuses a builtin name such as `exit` is that function, not the builtin, unless
its body unconditionally exits. `unset -f name` drops that exit proof. Plain
`unset name` drops it too unless `name` is a tracked array variable, because
with no variable Bash removes the function. `unset -v` removes only a variable
and keeps the proof. `set -u` and `set +u`
inside a function definition stay scoped to that body until the function is
called; a direct call to an earlier brace-body function that turns `nounset`
off on every returning path (or turns it on along any returning path) updates
the caller's `nounset` state at the call site, while a subshell function body
`f() ( set +u )` never changes the caller's `nounset` state.

A redirection can fail before a function runs. For a redirected call, the
rule accounts for both the function's effects and the path where the body
never runs. A redirection attached to the function definition is part of
every call, so that call keeps the same failure path. An unconditional
direct call can establish that an array is nonempty before a later expansion.
A redirection on `if`, `case`, or a loop keeps the path where that compound
does not run.

When an array starts empty and is populated inside an `if` / `elif` / `else`
branch or an `&&` list, the rule correlates that fact with later branches
guarded by the lexically identical condition (or an `&&` conjunction that
includes it), provided neither the array nor any variable referenced in the
condition is written in between (including arithmetic writes such as
`(( MODE++ ))`, `let "MODE=0"`, `$(( ++MODE ))`, and an assignment inside a
case pattern). Conditions correlate only for `[[`, `[`, `test`, and
arithmetic tests whose parameter expansions name their inputs directly. A
command predicate, an indirect expansion such as `${!name}`, or a file test
such as unary `-a` does not correlate, because the result can change with no
tracked write. When an array is
nonempty on some paths and empty on others, and the expansion is inside a
guarding condition that the rule does not model (such as a correlated flag
`if [[ $m == x ]]`, a different or weakened function test such as setting under
`if contains build || contains test` and reading under `if contains build`, or
a `case` arm), the rule reports the expansion through the `unanalyzable` policy
(`advise` by default) rather than guessing a definite error or silently
passing. Expanding an array unconditionally when an empty path reaches the read
— for example, resetting `arguments=()` in a loop, appending only inside one
`if` branch, and expanding `"${arguments[@]}"` after `fi`, or expanding an
array that is empty on every path reaching the read — remains a definite
finding.

## Analysis

### Unanalyzable inputs

When the analyzer encounters a control-flow or scoping shape it does not
soundly model, it abstains rather than issuing a silent pass or a guessed
finding. Abstentions are reported with `target: "unanalyzable"` under the
rule's `unanalyzable` policy (`advise` by default, or `report` / `ignore` via
`options.unanalyzable` or top-level `unanalyzable`):

- **Function redefinitions**: redefining a function name (for example replacing
  a brace-body helper with a subshell body) clears any earlier summary and
  emits an `unanalyzable` diagnostic.
- **Unmodeled inline subshell or group syntax**: inline subshells attached to
  unmodeled prefixes (such as `time ( :; exit 1 )`) isolate the subshell and
  emit an `unanalyzable` diagnostic.
- **Loop widening**: if a loop's array dependencies do not converge within the
  bounded pass limit, unsettled written arrays widen to `unanalyzable` rather
  than silently passing or producing a definite error on unrelated refilled
  arrays.
- **Correlated guarding conditions**: when an array is populated on some paths
  and empty on others, and the expansion sits inside an unmodeled guarding
  condition (`if` / `elif` / `else`, `&&` / `||`, or a `case` arm) that may
  exclude the empty paths, the expansion is reported through `unanalyzable`
  rather than as a definite finding.
- **File-wide loop replay budget**: loop replays share a budget across the
  script. If a loop still needs another replay when the budget is exhausted,
  the rule reports uncertainty at its `done`. Earlier findings are preserved,
  and array facts that the loop could invalidate are weakened before analysis
  continues past the loop.
- **Dynamic `unset` operands**: `unset` with a dynamic target (such as
  `unset "$name"`) clears nonempty guarantees on tracked arrays and widens them
  to `unanalyzable` until the array is refilled or guarded. When the `unset`
  operand is a variable reached only through literal-only `case` arms matching
  that variable (or a direct copy or Bash case transform such as
  `normalized="${name,,}"` with `case "$normalized" in alpha|beta|gamma_dir)`),
  the rule resolves the `unset` to that finite set of names so unrelated arrays
  stay proved. Other derived expressions (such as `"${target}_suffix"` or
  parameter trimming) remain dynamic `unset` targets.
- **Unmatched loop terminators**: an unmatched `done` without an enclosing loop
  emits an `unanalyzable` diagnostic rather than popping another frame.

C-style arithmetic `for (( init; cond; step ))` loops are modeled as loops
(including when nested in `while` or other loops) and track arithmetic writes
from the header without emitting spurious subshell advisories.

Rewrite the script to a direct static shape or guard the expansion with
`${name[@]+...}` (or suppress with a reason).

Scripts embedded in a workflow `run:` block are not scanned. The rule reads
files selected by `options.files`. Move the script to a file and list it there
to check it.

The check is lexical. It uses source order and skips nested substitutions and
here-document bodies, including bodies introduced with ANSI-C quoted
delimiters. It keeps combined output redirects and adjacent file descriptor
prefixes in the same command. It follows the direct length guards above
within their lexical scope. A direct call to an earlier function definition
can invalidate facts about tracked arrays written by that function. Calls
through dynamic names and sourced files are outside the analysis.

## Suppression

Use `# no-mistakes-disable-next-line bash-empty-array-nounset -- <reason>`
above the expansion, `# no-mistakes-disable-line bash-empty-array-nounset -- <reason>`
on the same line, or `# no-mistakes-disable-file bash-empty-array-nounset -- <reason>`
for a whole file. Explain the invariant that guarantees the expansion is safe.

## Related rules

[`zsh-readonly-parameter`](zsh-readonly-parameter.md) covers writes to zsh's
exit status. [`shellcheck-runner`](shellcheck-runner.md) integrates ShellCheck
diagnostics with `check`.
