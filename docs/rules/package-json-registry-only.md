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
way; omitted values use the documented registry-only defaults.
A pnpm lockfile check reads `packages` in every YAML document, so a non-registry
config dependency in the env document is reported along with project packages.
A lockfile that cannot be parsed, or whose leading document is not an env
lockfile, is reported instead of passing with no findings. A configured lockfile
path that does not exist is skipped.

```yaml
rules:
  - rule: package-json-registry-only
    scope: repository
```

## Valid example

A dependency that resolves from the npm registry passes.

```json
// package.json
{ "name": "@app/web", "version": "1.0.0", "dependencies": { "left-pad": "^1.3.0" } }
```

## Counterexample

`file:`, `link:`, `git+https:`, and direct tarball-style specifiers are findings.

```json
// package.json
{ "name": "@app/web", "version": "1.0.0", "dependencies": { "left-pad": "file:../vendor/left-pad" } }
```

## Fix

Use npm registry versions, `workspace:`, `catalog:`, or supported `npm:`
aliases; keep lockfile package entries registry-backed.

## Suppression

Package manifests cannot contain directives. Narrow `scopes`, `lockfile`, or
the rule application for an intentional exceptional registry source.

## Related rules

[`lockfile-allowlist`](lockfile-allowlist.md) limits lockfile formats and
[`pnpm-release-age-policy`](pnpm-release-age-policy.md) governs pnpm policy.
