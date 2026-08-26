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
exceptions must identify a package version, reason, and removal date.

## Options and defaults

`permanentPackages`, `temporaryGroups`, `temporarySelectors`, and
`scopedPrefixes` default to empty. `workspaceYaml`, `dependabotPath`, and
`lockfilePath` default to the repository's pnpm workspace, Dependabot config,
and lockfile paths when discoverable. `temporarySelectors` is a compatibility
flat list; selectors must not repeat across it and `temporaryGroups`.

## Valid example

```yaml
temporaryGroups:
  - selectors: ["acme-parser@2.4.1"]
    reason: upstream security release
    eligibleForRemovalAt: "2027-03-01T00:00:00Z"
```

## Counterexample

```yaml
minimumReleaseAgeExclude: [acme-parser]
```

The package is excluded from pnpm aging but absent from the declared permanent
or temporary policy.

## Fix

Add the package to the appropriate permanent or temporary declaration, flatten
the resulting selectors into pnpm settings, and mirror permanent names in
Dependabot.

## Suppression

Use `no-mistakes-disable-file pnpm-release-age-policy` only for a repository
whose release tooling is intentionally external; keep the external policy link
in the suppression comment.

## Related rules

[`pnpm-overrides-ban`](pnpm-overrides-ban.md) prevents version forcing, and
[`version-pin-consistency`](version-pin-consistency.md) checks other structured
version sources against their anchors.
