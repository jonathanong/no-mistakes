# `no-mistakes resolve-check`

Check whether every import in a single file resolves.

```sh
no-mistakes resolve-check src/new-feature.test.ts --format json
```

Use this right after writing or moving a file to confirm its imports point at real
modules. It is fully local — it parses only the target file and resolves each
specifier — so it returns in well under a second.

Each import is classified `resolved` (points at a local file), `external` (a bare
npm package, Node builtin, or subpath import), or `unresolved` (a relative or
aliased import whose target is missing). The command exits non-zero when any
import is unresolved, and lists the offending specifiers under `unresolved`.

A configured tsconfig path alias whose target is missing counts as `unresolved`,
not `external`. Pass `--tsconfig` in a monorepo so aliases resolve correctly. A
catch-all mapping (`"*": [...]`) matches every bare specifier, so under one a bare
npm package whose fallback target is absent is reported `unresolved` rather than
`external`. Conversely, with only `baseUrl` set (no `paths`), a missing
`baseUrl`-relative import like `src/typo` is indistinguishable from a bare npm
package and is reported `external` rather than `unresolved`. If the file itself
has a syntax error, the parser recovers and checks whatever imports it can still
read, so a malformed file may report `allResolve: true` from a partial list.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `resolveCheck(options)`.
