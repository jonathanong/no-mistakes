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

## Related rules

[`package-json-registry-only`](package-json-registry-only.md) controls package
sources, while [`pnpm-overrides-ban`](pnpm-overrides-ban.md) controls pnpm
resolution overrides.

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
