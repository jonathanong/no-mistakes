# `pnpm-release-age-policy`

Flags drift between pnpm `minimumReleaseAgeExclude`, configured permanent
packages plus temporary selectors, Dependabot npm `cooldown.exclude`, and the
active package graph. Registries are YAML options, not TypeScript files.

Empty options report nothing.

```yaml
rules:
  - rule: pnpm-release-age-policy
    scope: repository
    options:
      permanentPackages:
        - name: acme-lib
          reason: first-party published package
        - name: '@acme/core'
          reason: first-party published package
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

Counterexample: a first-party package missing from `minimumReleaseAgeExclude`.

```yaml
minimumReleaseAgeExclude:
  - acme-lib
```

Each `temporaryGroups` entry requires at least one exact `package@version`
selector, a non-empty reason, and a canonical UTC
`eligibleForRemovalAt` timestamp (`YYYY-MM-DDTHH:mm:ssZ`). The timestamp is
audit metadata, so an elapsed date does not fail CI. `temporarySelectors`
remains supported for compatibility, but a selector may appear only once across
the flat list and all groups.

Fix: keep the exclude list equal to `permanentPackages` plus flattened
temporary selectors, cover permanent names in Dependabot npm
`cooldown.exclude`, and keep those names in manifests or the lockfile.

## Why and when

Use this rule when a workspace delays newly published packages to reduce supply
chain risk but still needs an auditable exception process for first-party and
temporary releases.

## What it catches/requires

The configured policy must agree across pnpm workspace settings, Dependabot
cooldown exclusions, package manifests, and the active lockfile. Temporary
exceptions must identify a package version, reason, and removal date. A local
`include` that owns `minimumReleaseAgeExclude` supplies that list. Patterns
are not rebased; the entries are package selectors. When the root workspace
file and the included file both claim the setting, the check reports the
conflict and does not compare either list to the policy.

## Options and defaults

`permanentPackages`, `temporaryGroups`, `temporarySelectors`, and
`scopedPrefixes` default to empty. `workspaceYaml`, `dependabotPath`, and
`lockfilePath` default to the repository's pnpm workspace, Dependabot config,
and lockfile paths when discoverable. `temporarySelectors` is a compatibility
flat list; selectors must not repeat across it and `temporaryGroups`.

## Valid example

Permanent packages, flattened temporary selectors, Dependabot cooldown
exclusions, and lockfile keys agree.

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - acme-lib
  - '@acme/core'
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
{ "dependencies": { "acme-lib": "1.0.0", "@acme/core": "1.0.0" } }
```

```yaml
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  'acme-lib@1.0.0':
    resolution: {}
  '@acme/core@1.0.0':
    resolution: {}
  'demo-temporary-package@9.9.9':
    resolution: {}
```

## Counterexample

```yaml
# pnpm-workspace.yaml
minimumReleaseAgeExclude:
  - acme-lib
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
{ "dependencies": { "acme-lib": "1.0.0", "@acme/core": "1.0.0" } }
```

```yaml
# pnpm-lock.yaml
lockfileVersion: '9.0'
packages:
  'acme-lib@1.0.0':
    resolution: {}
  '@acme/core@1.0.0':
    resolution: {}
  'demo-temporary-package@9.9.9':
    resolution: {}
```

`@acme/core` and the temporary selector are missing from
`minimumReleaseAgeExclude`.

## Fix

Add the package to the appropriate permanent or temporary declaration, flatten
the resulting selectors into pnpm settings, and mirror permanent names in
Dependabot.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, the intent is that nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)). Known gap: invalid workspace YAML is already reported, but an unreadable pnpm workspace file and an unparseable or unreadable `package.json` are currently skipped without a finding ([#2659](https://github.com/jonathanong/no-mistakes/issues/2659)).

## Suppression

Use `no-mistakes-disable-file pnpm-release-age-policy` only for a repository
whose release tooling is intentionally external; keep the external policy link
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
