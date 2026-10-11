# `no-mistakes dead-exports`

Check whether any files still import the given exports — a fast yes/no before
deleting code.

```sh
no-mistakes dead-exports src/utils.mts --format json
no-mistakes dead-exports src/utils.mts oldHelper legacyFn --format json
```

With no names, every export of the file is checked. With names, only those are
checked (the name need not still be an export, so this works before _and_ after a
deletion). Each result reports `referenced` and an `importerCount`; the command
exits non-zero when any checked export is dead, which is convenient in CI and
agent loops.

References are counted from import edges, including re-export barrels (named and
`export *`), namespace imports (`import * as ns`), and default imports. Dynamic or
string-keyed access (`obj["fn"]`) and inline import-type references
(`type T = import('./x').Foo`) are not detected, so a "dead" verdict means "no
file imports this symbol." Wildcard edges are counted conservatively: an
`export *` barrel or `import * as ns` that may forward a symbol keeps it
"referenced" even if that specific symbol is shadowed or unused through the
barrel — `dead-exports` favors a false "referenced" over a false "dead".
Consumers that import through a recognized **workspace package name** such as
`@scope/pkg` are resolved through the workspace package's visible entry points,
so cross-package imports can keep exports alive. A configured tsconfig `paths`
alias or `baseUrl` resolution takes precedence and can resolve a subpath even
when the package's `exports` blocks it. A missing or blocked subpath creates no
edge when workspace resolution handles it. Third-party packages are outside
this query's project-wide reverse index.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `deadExports(options)`.

The reverse scan adds `partial` and `incompleteFiles` to structured output.
Each incomplete file reports a root-relative `file`, a `reason`
(`operational-error`, `parse-error`, `missing-facts`, or `missing-symbols`), and a
`message`. The list covers the prepared TS/JS candidate inventory: a failed file
may contain an import that cannot be recovered, so its relevance can be unknown.
Recovered imports may still contribute results when a parser diagnostic exists.
`partial: false` means no recorded collection gaps; it does not guarantee that
all runtime references or unsupported syntax were resolved. When `partial` is
true, treat empty lists and zero counts as known results, not proof of absence.
Human output prints the same warnings. `--format paths` retains the path list
without diagnostics; use JSON before making decisions that depend on absence.

`anyDead`, `referenced`, importer counts, and exit status retain their existing
meaning. A partial report with `anyDead: true` is not sufficient evidence to
delete an export; repair the listed files and rerun the query first.
