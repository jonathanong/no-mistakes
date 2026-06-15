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
not `external`. Pass `--tsconfig` in a monorepo so aliases resolve correctly.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `resolveCheck(options)`.
