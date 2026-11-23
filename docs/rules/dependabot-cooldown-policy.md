# `dependabot-cooldown-policy`

## Why and when

An entry without a cooldown can propose updates on the day they are published.
Enable this rule to apply a consistent waiting period as ecosystems are added.

## What it requires

Each update entry in `.github/dependabot.yml` or `.github/dependabot.yaml`
must set a default cooldown at the configured floor. If both files exist, both
are checked. Explicit SemVer cooldown values use the same floor.

## Options and defaults

```yaml
rules:
  - rule: dependabot-cooldown-policy
    scope: repository
    options:
      minimumDays: 7
      requireOpenPullRequestsLimit: true
      allowedCooldownExcludes:
        - "@example/*"
      exemptEcosystems:
        - docker
```

| Option | Behavior |
| --- | --- |
| `minimumDays` | Required nonnegative integer. Each checked entry must set `cooldown.default-days` to at least this value. Any explicit `semver-major-days`, `semver-minor-days`, or `semver-patch-days` in that cooldown must meet the same floor. |
| `requireOpenPullRequestsLimit` | Defaults to `false`. Set to `true` to require an explicit nonnegative integer `open-pull-requests-limit` on every checked entry. Zero is valid. |
| `allowedCooldownExcludes` | Optional list of whole-value globs for entries in `cooldown.exclude`. An omitted list allows any exclusion. An empty list allows none. `*` can match across `/`. |
| `exemptEcosystems` | Optional list of exact `package-ecosystem` names. Matching entries skip every policy check. The default list is empty. |

The rule is silent when neither file exists. When either selected file cannot
be read or parsed, or its `updates` structure cannot be analyzed, the JSON report records an
`incomplete` entry with this rule as `consumer`, the file path, and a repair message.
`check` exits unsuccessfully for incomplete analysis.

## Valid example

With the options above, this entry is compliant. Both day values meet the floor,
the explicit PR limit is valid, and the exclusion matches the allowlist.

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: npm
    directory: /
    schedule: { interval: weekly }
    open-pull-requests-limit: 3
    cooldown:
      default-days: 7
      semver-major-days: 14
      exclude: ["@example/internal"]
```

## Counterexample

This entry omits the PR limit, sets two day values below seven, and excludes a
package outside the allowlist.

```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: npm
    directory: /
    schedule: { interval: weekly }
    cooldown:
      default-days: 3
      semver-major-days: 0
      exclude: ["external-package"]
```

## Fix

Add `cooldown.default-days` at the required floor to every applicable entry.
Raise each shorter explicit SemVer cooldown to the floor or remove the
override to use the default. When enabled, add the missing PR limit and remove
exclusions that the allowlist does not cover.

## Suppression

Use a YAML `# no-mistakes-disable-line dependabot-cooldown-policy` on the
reported line, or `# no-mistakes-disable-next-line dependabot-cooldown-policy`
immediately above it. Use a leading
`# no-mistakes-disable-file dependabot-cooldown-policy` for a file-wide
exception. For an ecosystem-wide exception, configure `exemptEcosystems`.

See [check](../cli/check.md) for CLI usage and [the Node API](../node-api.md)
for asynchronous checks.

## Related rules

[`pnpm-release-age-policy`](pnpm-release-age-policy.md) checks pnpm release-age
configuration. [`finite-set-consistency`](finite-set-consistency.md) compares
values across sources, such as mirroring cooldown exclusions in a registry list.
