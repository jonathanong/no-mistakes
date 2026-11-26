# `tsconfig-file-coverage`

Requires every tracked TypeScript file (`.ts`, `.mts`, `.cts`, `.tsx`) to be
owned by a tracked tsconfig, or by a reasoned `allow` entry. Ownership is an
explicit `files` entry, a matching `include`, or, when both keys are absent,
TypeScript's implicit include of sources under that config directory, minus
`exclude`. `files` is exclusive only when `include` is absent. This is the
file-universe check: it does not inspect CI or local `tsc` registration. Use
[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) for that.

The filesystem dispatcher supplies the Git-tracked inventory when a discovery
snapshot is available, so untracked scratch files are ignored. Direct callers
without a snapshot should pass tracked paths themselves.

```yaml
rules:
  - rule: tsconfig-file-coverage
    scope: repository
```

Optional `allow` and `auxiliaryConfigs` entries name reasoned exceptions.
Empty `reason` values and absolute or `..` paths are findings for both. An
`auxiliaryConfigs` path that is not a tracked tsconfig is a stale finding. A
missing `allow` path is not a `tsconfig-file-coverage` finding; it is an
`unused-allow-entry` advisory when `reportUnusedAllowEntries` is `advise`
(the default) or `report`:

```yaml
rules:
  - rule: tsconfig-file-coverage
    scope: repository
    options:
      allow:
        - path: scripts/generate.ts
          reason: generated entrypoint kept outside the app program
      auxiliaryConfigs:
        - path: tsconfig.dependency-cruiser.json
          reason: dependency-cruiser resolver config
      requiredBasename: tsconfig.dependency-cruiser.json
```

Tracked `tsconfig.json` / `tsconfig.*.json` files are the candidate programs,
including tsconfigs that a rule `include` filter would otherwise drop. Files
under `node_modules`, `dist`, `.git`, `.next`, `coverage`, `fixtures`,
`target`, or `build` are out of scope. Repositories with no tracked tsconfig
produce no source-coverage findings, so fixtures without compiler config stay
silent.
Configured `allow` and `auxiliaryConfigs` entries are still validated.
`auxiliaryConfigs` entries are not compiler programs: they must not declare
`files`, `include`, `exclude`, or `references`, and their basename defaults to
`tsconfig.dependency-cruiser.json`.

Coverage uses `TsConfigCatalog::project_source_union` and the same
`files`/`include`/`exclude` matcher as `project_source_membership`. It does
not close over the static import graph, so a file imported only by a program
member is still uncovered unless that file is itself owned by a tsconfig.

Counterexample: `orphan.ts` sits next to a program that only includes `src`.

```json
{
  "include": ["src"]
}
```

```ts
export const orphan = 1;
```

Fix: add the file to a tsconfig `include`/`files` set, or give it a reasoned
`allow` entry.

```yaml
allow:
  - path: orphan.ts
    reason: generated script excluded from the app program
```

Findings use line 1 of the uncovered file or option path. Use a top-of-file
`no-mistakes-disable-file tsconfig-file-coverage` directive, or
`no-mistakes-disable-line` on line 1, for a one-off exception.

## Why and when

Use this rule when every tracked TypeScript source file must belong to a
compiler program, independent of whether CI happens to run that program.

## What it catches/requires

Every tracked `.ts`, `.mts`, `.cts`, and `.tsx` file must be owned by at least
one tracked tsconfig or a reasoned `allow` entry, using the ownership rules
above.

## Options and defaults

`allow` and `auxiliaryConfigs` default to empty. `requiredBasename` defaults to
`tsconfig.dependency-cruiser.json` for auxiliary configs. Auxiliary configs may
not declare compiler `files`, `include`, `exclude`, or `references`; paths must
be repository-relative and reasons non-empty.

## Valid example

```json
// tsconfig.json
{"include":["src/**/*.ts"]}
```

```ts
// src/index.ts
export const ok = 1;
```

## Counterexample

```json
// tsconfig.json
{"include":["src"]}
```

```ts
// orphan.ts
export const orphan = 1;
```

The file is tracked beside a config that includes only `src`.

## Fix

Add the file to a compiler `include`/`files` set or add a precise `allow` entry
with why it intentionally stays outside the program.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

An unreadable auxiliary config, a parse failure, or a top-level value other
than a JSON object produces an ordinary finding. The rule does not use
`options.unanalyzable`. [Auxiliary config validation](https://github.com/jonathanong/no-mistakes/blob/cc505a7c1e9c3cb6ad6d69cce53633b67d0c999e/crates/no-mistakes/src/codebase/rules/tsconfig_file_coverage/lists.rs#L109-L129).

## Suppression

Prefer a reasoned `allow` entry. Use a top-of-file file directive for a
one-off generated file; findings point to line 1.

## Limitations

Discovery skips any directory segment named `node_modules`, `dist`, `.git`,
`.next`, `coverage`, `fixtures`, `target`, or `build`. A tracked file under
one of those names is not visible, so it is not reported as uncovered.
`node_modules` was already out of scope; the other names are skipped the same
way.

```text
fixtures/orphan.ts
src/build/orphan.ts
```

Next to a tsconfig whose `include` is `["src"]`, neither file is a finding.

## Related rules

[`tsconfig-gate-coverage`](tsconfig-gate-coverage.md) checks CI and local
typecheck registration; [`tsconfig-alias-folder-mapping`](tsconfig-alias-folder-mapping.md)
checks alias targets.

A missing `allow` path is not a finding of this rule. See the
`unused-allow-entry` note below and
[allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).

With `reportUnusedAllowEntries: report` or `advise`, `allow[].path` also
produce `unused-allow-entry` when the literal path is absent from both the
request inventory and disk, within the application's configured scope. Remove
the entry or fix its path. Existing excluded or ignored files and empty glob
matches stay silent; narrowed runs do not audit paths outside their scope.
See [allow-entry hygiene](../configuration/suppressions.md#allow-entry-hygiene).

## Invalid UTF-8

An auxiliary tsconfig is decoded lossily, so a program key next to a stray byte is still reported. A file that cannot be read for any other reason is reported as not a JSON object.
