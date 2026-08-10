# `no-mistakes resolve-check`

Check whether every import in one or more files resolves.

```sh
no-mistakes resolve-check src/new-feature.test.ts --format json
no-mistakes resolve-check src/new-feature.test.ts src/changed-module.ts --format json
```

Use this right after writing or moving a file to confirm its imports point at real
modules. It is fully local — it parses only the target file and resolves each
specifier — so it is typically fast (well within the sub-5-second target).
For multiple files it discovers the visible project once, collects the union of
requested import facts once, and classifies files in parallel.

A single file preserves the existing result object. Multiple files return
`{ allResolve, unresolvedFiles, results }`; `results` and `unresolvedFiles` are
sorted and duplicate input paths are checked once. `--format paths` prints the
sorted, deduplicated union of resolved local targets. Human and Markdown output
use a separate block for each file. Any unresolved import exits 1; an invalid
input or operational error exits 2 without a partial report.

Each import is classified `resolved` (points at a local file), `external` (a bare
npm package, Node builtin, or subpath import), or `unresolved` (a relative or
aliased import whose target is missing). The command exits non-zero when any
import is unresolved, and lists the offending specifiers under `unresolved`.

A configured tsconfig path alias whose target is missing counts as `unresolved`,
not `external`. In a workspace, the omitted default selects the config owning
the checked file; pass `--tsconfig` only to force a particular config. A
catch-all mapping (`"*": [...]`) matches every bare specifier, so under one a bare
npm package whose fallback target is absent is reported `unresolved` rather than
`external`. Conversely, with only `baseUrl` set (no `paths`), a missing
`baseUrl`-relative import like `src/typo` is indistinguishable from a bare npm
package and is reported `external` rather than `unresolved`. Emitted `.js`/`.mjs`/`.cjs` specifiers resolve to their `.ts`/`.mts`/`.cts`
sources (NodeNext/ESM), and a type-only import resolves declaration (`.d.ts`)
modules. TypeScript `import x = require()` declarations are not checked. If the
file itself has a syntax error, the parser recovers and checks whatever imports
it can still read, so a malformed file may report `allResolve: true` from a
partial list.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `resolveCheck({ file, ... })` returns the single-file result;
`resolveCheck({ files, ... })` always returns the batch result, including for a
one-element `files` array. Provide exactly one of `file` or a nonempty `files`
array.
