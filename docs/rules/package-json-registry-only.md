# `package-json-registry-only`

## Why and when

Use this rule when package declarations must resolve from approved registries
instead of Git, local paths, or unapproved URLs.

## What it catches

It reports package and lockfile sources that do not match the configured scope
registry policy.

## Options

`scopes` maps package scopes to allowed registries and `lockfile` chooses the
checked lockfile; omitted values use the documented registry-only defaults.

## Valid example

A scoped dependency and its lockfile resolution both pointing at the configured
registry pass.

## Related rules

[`lockfile-allowlist`](lockfile-allowlist.md) limits lockfile formats and
[`pnpm-release-age-policy`](pnpm-release-age-policy.md) governs pnpm policy.

## Suppression

Package manifests cannot contain directives. Narrow `scopes`, `lockfile`, or
the rule application for an intentional exceptional registry source.

Requires package registry settings to match configured policy.

```yaml
rules:
  - rule: package-json-registry-only
    scope: repository
    options:
      scopes: [packages]
      lockfile: pnpm-lock.yaml
```

Counterexample: `package.json` using `file:`, `link:`, `git+https:`, or direct
tarball-style dependency specifiers.

Fix: use npm registry versions, `workspace:`, `catalog:`, or supported
`npm:` aliases; keep lockfile package entries registry-backed.
