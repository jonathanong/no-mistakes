# `ast-grep-ignore-freshness`

```yaml
rules:
  - rule: ast-grep-ignore-freshness
    scope: repository
    options:
      files:
        - sgconfig.yml
```

## Why and when

Ast-grep `ignores` entries are path globs. A literal that names no tracked file,
or a glob that matches no tracked path, is stale configuration. The check is
opt-in and compares each entry with the tracked file inventory. It does not
parse source text, and a finite string-set comparison cannot decide whether a
wildcard matches some inventory path.

## What it catches

Each string ignore, and each `{glob, caseInsensitive}` object, must match at
least one tracked path. A glob matches with ast-grep's glob rules: `*` can
cross `/`, and a dotfile path matches. `!` is a literal character, not a
negated glob. Ast-grep strips a leading `./` from the file path and matches
the ignore text as written, so `./src/items.ts` does not match `src/items.ts`.

Paths are relative to the directory of the `sgconfig` file that discovered the
rule. A root `sgconfig.yml` uses repository paths. With
`files: [packages/widgets/sgconfig.yml]`, `src/items.ts` matches
`packages/widgets/src/items.ts` and not a repository-root `src/items.ts`. A
rule file named directly in `files`, with no `sgconfig`, stays relative to the
repository root. When `files` also lists the `sgconfig` that owns that rule
directory, the `sgconfig` directory still wins.

A rule file may contain several YAML documents separated by `---`. Each
document's ignores are checked. A document that is not valid YAML is a
finding. Documents parsed before that error are still checked. A flow list
such as `ignores: ['src/missing.ts', 'src/items.ts']` reports the line that
contains the entry, including when the list continues on later lines. A
comment after `ignores:` does not move the entry, and a `-` item at the same
indent as `ignores:` is that entry. A `no-mistakes-disable-next-line`
directive on the previous line applies to that entry. The same text on a
`pattern:` line is not an ignore entry and does not move that line.

An entry that matches only an untracked file or only a gitignored file is
still reported. Inside a Git worktree the inventory is the tracked set,
including tracked files under source skip directories such as `fixtures/`.
Outside Git, the rule uses the ignore-aware visible inventory.

`files` names the configs to read. Each entry is a tracked rule YAML file, a
directory of tracked rule YAML, or a tracked `sgconfig` file whose `ruleDirs`
are scanned. When `files` is empty, the rule reads root `sgconfig.yml` or
`sgconfig.yaml` and the tracked YAML under its `ruleDirs`.

Selected config source and structured YAML documents are reused from the
request-scoped source store and index. This avoids a separate source read and
YAML parse for the rule; selection, parsing errors, and freshness findings
keep the same semantics.

## Options

`files` defaults to empty, which selects root `sgconfig.yml` / `sgconfig.yaml`.

## Valid example

`src/items.ts` and `src/orders/widget.ts` are tracked, so both ignores match.

```yaml
# sgconfig.yml
ruleDirs:
  - rules

# rules/widgets.yml
id: list-widgets
language: TypeScript
rule:
  pattern: listWidgets()
ignores:
  - src/items.ts
  - 'src/orders/**/*.ts'

# src/items.ts
export const items = [];

# src/orders/widget.ts
export const widget = [];
```

## Counterexample

```yaml
# sgconfig.yml
ruleDirs:
  - rules

# rules/widgets.yml
id: list-widgets
language: TypeScript
rule:
  pattern: listWidgets()
ignores:
  - src/missing.ts
  - 'retired/**/*.ts'
```

## Fix

Delete the stale ignore entry, or restore a tracked file that matches it.
Creating an untracked or gitignored file does not satisfy the entry. A dotfile
path such as `.widgets/items.ts` is a normal tracked path when Git tracks it.
Remove a leading `./` from an ignore when the tracked path does not include
it. Write paths relative to the `sgconfig` directory, not the repository root,
when that config is nested.

## Suppression

Use `no-mistakes-disable-next-line ast-grep-ignore-freshness` on the ignore
entry, or `no-mistakes-disable-file ast-grep-ignore-freshness` at the top of a
rule file.

```yaml
# rules/widgets.yml
ignores:
  # no-mistakes-disable-next-line ast-grep-ignore-freshness
  - src/missing.ts
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)). Malformed YAML, an unreadable config, an ignore that is not a string or `{glob, caseInsensitive}` mapping, and an invalid glob are findings. `options.unanalyzable` has no effect.

## Related rules

[`config-path-references`](config-path-references.md) checks configured path
strings, including the `sgconfig` preset's `ruleDirs`. It does not check
per-rule `ignores`. [`finite-set-consistency`](finite-set-consistency.md)
compares equal string sets and does not test a wildcard against the inventory.
See [discovery](../configuration/discovery.md) for the tracked inventory.
