# `pnpm-release-age-policy`

Flags drift between pnpm `minimumReleaseAgeExclude`, the exemption registries
in the rule's YAML options, Dependabot npm `cooldown.exclude`, and the active
package graph.

Empty options report nothing.

```yaml
rules:
  - rule: pnpm-release-age-policy
    scope: repository
    options:
      permanentPackages:
        - name: acme-lib
          reason: first-party published package
        - pattern: '@acme/*'
          reason: first-party published packages
      temporaryGroups:
        - selectors:
            - demo-temporary-package@9.9.9
          reason: upstream release regression
          eligibleForRemovalAt: '2027-01-02T03:04:05Z'
      scopedPrefixes:
        - '@acme/'
      workspaceYaml: pnpm-workspace.yaml
      dependabotPath: .github/dependabot.yml
      lockfilePath: pnpm-lock.yaml
```

Lockfile presence uses package keys from every YAML document in `pnpm-lock.yaml`.
A selector that appears only as a pnpm 12 config dependency still counts as present.
A lockfile that cannot be parsed, or whose leading document is not an env
lockfile, is reported instead of treating every package as absent.

Each `temporaryGroups` entry requires at least one exact `package@version`
selector, a non-empty reason, and a canonical UTC
`eligibleForRemovalAt` timestamp (`YYYY-MM-DDTHH:mm:ssZ`). The timestamp is
audit metadata, so an elapsed date does not fail CI. `temporarySelectors`
remains supported for compatibility, but a selector may appear only once across
the flat list and all groups.

## Why and when

Use this rule when a workspace delays newly published packages to reduce supply
chain risk but still needs an auditable exception process for first-party and
temporary releases.

## What it catches/requires

The configured policy must agree across pnpm workspace settings, Dependabot
cooldown exclusions, package manifests, and the active lockfile. Temporary
exceptions must identify a package version, reason, and removal date. A local
`include` that owns `minimumReleaseAgeExclude` supplies that list. When the
root workspace file and the included file both claim the setting, the check
reports the conflict and skips the policy comparison for that list.

### Package-name patterns

A `permanentPackages` entry takes either an exact `name` or a `pattern`,
with an optional `reason`. A pattern registers every matching package name in
the active manifests and lockfile. A pattern with no active match is reported.

The registry, pnpm `minimumReleaseAgeExclude`, and Dependabot npm
`cooldown.exclude` use the same package-name matcher. `*` matches any
sequence of characters, `?` matches one character, and a leading `!`
negates that entry. All other characters are literal. The matcher evaluates
each entry independently against the whole package name, including any scope
and its `/`. Quote YAML patterns.

Either a literal name or a matching glob in each exclusion list can cover a
registered permanent package. An exclusion glob is reported if it covers no
registered permanent package or matches any lockfile package outside the
permanent registry. Temporary entries still require exact `package@version`
selectors in pnpm. A temporary selector authorizes only that package version.

## Options and defaults

`permanentPackages`, `temporaryGroups`, `temporarySelectors`, and
`scopedPrefixes` default to empty. `workspaceYaml`, `dependabotPath`, and
`lockfilePath` default to the repository's pnpm workspace, Dependabot config,
and lockfile paths when discoverable. `temporarySelectors` is a compatibility
flat list; selectors must not repeat across it and `temporaryGroups`.

## Valid example

The `@acme/*` pattern covers both locked `@acme` packages. The exact
permanent and temporary entries cover the other exclusions.

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - acme-lib
  - '@acme/*'
  - demo-temporary-package@9.9.9
```

```yaml
# .github/dependabot.yml
updates:
  - package-ecosystem: npm
    directory: /
    cooldown:
      exclude:
        - acme-lib
        - '@acme/*'
```

```json
// package.json
{ "dependencies": { "acme-lib": "1.0.0", "@acme/core": "1.0.0", "@acme/cli": "1.0.0" } }
```

```yaml
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  'acme-lib@1.0.0':
    resolution: {}
  '@acme/core@1.0.0':
    resolution: {}
  '@acme/cli@1.0.0':
    resolution: {}
  'demo-temporary-package@9.9.9':
    resolution: {}
```

## Counterexample

With the same registry and lockfile, this pnpm list exempts
`demo-temporary-package` for every version through `demo-*`. The registry
authorizes only `demo-temporary-package@9.9.9`. Dependabot also leaves
`@acme/cli` uncovered.

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - acme-lib
  - '@acme/*'
  - 'demo-*'
```

```yaml
# .github/dependabot.yml
updates:
  - package-ecosystem: npm
    directory: /
    cooldown:
      exclude:
        - acme-lib
        - '@acme/core'
```

```json
// package.json
{ "dependencies": { "acme-lib": "1.0.0", "@acme/core": "1.0.0", "@acme/cli": "1.0.0" } }
```

```yaml
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  'acme-lib@1.0.0':
    resolution: {}
  '@acme/core@1.0.0':
    resolution: {}
  '@acme/cli@1.0.0':
    resolution: {}
  'demo-temporary-package@9.9.9':
    resolution: {}
```

## Fix

Declare each permanent exemption with a package name or a pattern covering
only intended packages, then cover those packages in pnpm and Dependabot.
For the counterexample, use `demo-temporary-package@9.9.9` in pnpm and add
`@acme/cli` to Dependabot or use the registered `@acme/*` pattern.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

A single leading UTF-8 byte order mark in `package.json` is accepted.

Selected package manifests, workspace YAML, and present or explicitly configured Dependabot and lockfile inputs that cannot be read or parsed produce an `unanalyzable` diagnostic containing the file and error. Repair malformed JSON/YAML or restore read access so dependency policy can be checked. Absent optional inputs are allowed unless explicitly configured.

`options.unanalyzable` uses the shared analysis policy: `report` (default) emits a finding, `advise` emits an advisory, and `ignore` omits the uncertain result. Definite violations remain findings under every policy. Read/parse failures are file-level diagnostics at line 1; use a file suppression for these failures. File suppressions apply before policy routing; malformed JSON can carry a leading `// no-mistakes-disable-file pnpm-release-age-policy` directive, and YAML supports the corresponding `#` directive. Rule include/exclude and project scope still limit the selected files. A custom rule `message` prefixes these diagnostics.

## Suppression

Use `no-mistakes-disable-file pnpm-release-age-policy` for a repository
whose release tooling is intentionally external. Keep the external policy link
in the suppression comment.

## Related rules

[`pnpm-overrides-ban`](pnpm-overrides-ban.md) prevents version forcing, and
[`version-pin-consistency`](version-pin-consistency.md) checks other structured
version sources against their anchors.

## Limitations

Settings owned by a pnpm config dependency cannot be loaded without an install.
This rule does not read `node_modules`. A source whose key does not start with
`./` or `../` is ignored, so a `minimumReleaseAgeExclude` list that exists
only there is not the list this rule compares.

```yaml
include:
  "@acme/pnpm-policy":
    - minimumReleaseAgeExclude
```
