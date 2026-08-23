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
