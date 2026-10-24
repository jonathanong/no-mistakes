# `lockfile-allowlist`

## Why and when

Use this rule when one package-manager lockfile format is the repository
contract and accidental secondary lockfiles would make installs ambiguous.

## What it catches

It reports selected lockfile names not present in the allowlist.

## Options

`allowed` is the complete allowed path-pattern list and defaults to
`["pnpm-lock.yaml"]` when omitted or empty. `bannedBasenames` defaults to the
npm, Yarn, and Bun lockfile names; supplying a non-empty list replaces those
defaults.

## Valid example

A repository containing only its configured `pnpm-lock.yaml` passes.

```text
pnpm-lock.yaml
```


## Related rules

[`package-json-registry-only`](package-json-registry-only.md) controls package
sources, while [`pnpm-overrides-ban`](pnpm-overrides-ban.md) controls pnpm
resolution overrides.

## Counterexample

```text
package-lock.json
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](../../README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges lockfile names from the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Lockfile findings are file-level; use `no-mistakes-disable-file
lockfile-allowlist` only while migrating package managers. Prefer updating the
allowlist once the additional lockfile is intentional.

Allows only configured package-manager lock files.

```yaml
rules:
  - rule: lockfile-allowlist
    scope: repository
    options:
      allowed: [pnpm-lock.yaml]
```

Counterexample: adding `package-lock.json` to a pnpm workspace.

Fix: remove the unexpected lockfile or update the allowlist.
