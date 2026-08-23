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
      temporarySelectors:
        - demo-temporary-package@9.9.9
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

Fix: keep the exclude list equal to `permanentPackages` plus
`temporarySelectors`, cover permanent names in Dependabot npm
`cooldown.exclude`, and keep those names in manifests or the lockfile.
