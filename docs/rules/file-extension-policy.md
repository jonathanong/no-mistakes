# `file-extension-policy`

## Why and when

Use this rule when a directory has a deliberate language or generated-file
boundary that names alone cannot enforce.

## What it catches

It rejects a configured extension under a matching scope unless the exact path
is allowlisted; TypeScript declaration files are exempt.

## Options

`allowlist` is exact repository-relative paths. Each `scopes` entry has `path`
and `bannedExtensions`; no implicit scope or banned-extension default exists.

## Valid example

A `.ts` file under a scope banning `.js` and `.jsx` passes.

## Related rules

[`banned-paths`](banned-paths.md) bans specific path shapes and
[`banned-renamed-files`](banned-renamed-files.md) bans migration names.

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
