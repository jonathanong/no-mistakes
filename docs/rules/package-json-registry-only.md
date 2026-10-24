# `package-json-registry-only`

## Why and when

Use this rule when package declarations must resolve from approved registries
instead of Git, local paths, or unapproved URLs.

## What it catches

It reports package and lockfile sources that do not match the configured
registry policy.

## Options

`scopes` is an optional list of repository-relative directory prefixes. A
`package.json` is checked when `scopes` is empty or the file sits under one of
those prefixes. `lockfile` optionally names a pnpm lockfile to check the same
way; when omitted, only selected package manifests are checked.
A pnpm lockfile check reads `packages` in every YAML document, so a non-registry
config dependency in the env document is reported along with project packages.
A lockfile that cannot be parsed, or whose leading document is not an env
lockfile, is reported instead of passing with no findings. A configured lockfile
path that does not exist produces an `unanalyzable` diagnostic.

```yaml
rules:
  - rule: package-json-registry-only
    scope: repository
    options:
      scopes: [packages]
```

`scopes: [packages]` limits the check to manifests under `packages/`. Add
`lockfile: pnpm-lock.yaml` to also check that required lockfile.

## Valid example

A dependency under `packages/` that resolves from the npm registry passes.

```json
// packages/web/package.json
{ "name": "@app/web", "version": "1.0.0", "dependencies": { "left-pad": "^1.3.0" } }
```

## Counterexample

`file:`, `link:`, `git+https:`, and direct tarball-style specifiers are findings.

```json
// packages/web/package.json
{ "name": "@app/web", "version": "1.0.0", "dependencies": { "left-pad": "file:../vendor/left-pad" } }
```

## Fix

Use npm registry versions, `workspace:`, `catalog:`, or supported `npm:`
aliases; keep lockfile package entries registry-backed.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Selected package manifests and explicitly configured lockfiles that cannot be read or parsed produce an `unanalyzable` diagnostic containing the file and error. Repair malformed JSON/YAML or restore read access so dependency policy can be checked. Absent optional inputs are allowed unless explicitly configured.

`options.unanalyzable` uses the shared analysis policy: `report` (default) emits a finding, `advise` emits an advisory, and `ignore` omits the uncertain result. Definite violations remain findings under every policy. Read/parse failures are file-level diagnostics at line 1; use a file suppression for these failures. File suppressions apply before policy routing; malformed JSON can carry a leading `// no-mistakes-disable-file package-json-registry-only` directive, and YAML supports the corresponding `#` directive. Rule include/exclude and project scope still limit the selected files. A custom rule `message` prefixes these diagnostics.

## Suppression

For an intentional exception, narrow `scopes`, `lockfile`, or the rule
application. A malformed manifest can use the leading file directive described
above to suppress its read/parse diagnostic; lockfile YAML uses a `#` directive.

## Related rules

[`lockfile-allowlist`](lockfile-allowlist.md) limits lockfile formats and
[`pnpm-release-age-policy`](pnpm-release-age-policy.md) governs pnpm policy.
