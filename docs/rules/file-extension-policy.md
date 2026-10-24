# `file-extension-policy`

## Why and when

Use this rule when a directory has a deliberate language or generated-file
boundary that names alone cannot enforce.

## What it catches

It rejects a configured extension under a matching scope unless the exact path
is allowlisted; TypeScript declaration files are exempt. Scope paths collapse
`./`, empty segments, and in-repo `..`, so `./src` is `src`. A leading `/` is
repository-relative (`/src` is `src`). A scope that leaves the repository,
such as `../outside`, is one finding on the config file at line 1
(`../outside: configured root escapes the repository`) and matches no file.

## Options

`allowlist` is exact repository-relative paths. Each `scopes` entry has `path`
and `bannedExtensions`; no implicit scope or banned-extension default exists.

## Valid example

A `.ts` file under a scope banning `.js` and `.jsx` passes.

```ts
// src/app.ts
export const ok = 1;
```


## Related rules

[`banned-paths`](banned-paths.md) bans specific path shapes and
[`banned-renamed-files`](banned-renamed-files.md) bans migration names.

## Counterexample

```js
// src/legacy.js
export const bad = 1;
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, nothing is silently treated as clean ([how analysis works](README.md#how-analysis-works)); this rule has no unanalyzable inputs because it judges file extensions from the file inventory, so `options.unanalyzable` has no effect on it.

## Suppression

Findings are on the file at line 1, so use `no-mistakes-disable-file
file-extension-policy` only for an intentional exception. Prefer an exact
`allowlist` entry when the exception is part of the policy.

Enforces allowed or banned file extensions under configured scopes.

```yaml
rules:
  - rule: file-extension-policy
    scope: repository
    options:
      allowlist: ["src/generated/client.js"]
      scopes:
        - path: src
          bannedExtensions: [".js", ".jsx"]
```

Counterexample: adding `src/helper.js` where only TypeScript is allowed.

Fix: rename or move the file, or adjust the policy intentionally.
