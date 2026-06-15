# `no-mistakes dead-exports`

Check whether any files still import the given exports — a fast yes/no before
deleting code.

```sh
no-mistakes dead-exports src/utils.mts --format json
no-mistakes dead-exports src/utils.mts oldHelper legacyFn --format json
```

With no names, every export of the file is checked. With names, only those are
checked (the name need not still be an export, so this works before *and* after a
deletion). Each result reports `referenced` and an `importerCount`; the command
exits non-zero when any checked export is dead, which is convenient in CI and
agent loops.

References are counted from import edges, including re-export barrels (named and
`export *`), namespace imports (`import * as ns`), and default imports. Dynamic or
string-keyed access (`obj["fn"]`) and inline import-type references
(`type T = import('./x').Foo`) are not detected, so a "dead" verdict means "no
file imports this symbol."

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `deadExports(options)`.
