# `no-mistakes/no-banned-import-outside-allowed-paths`

Disallows configured module exports (or calls on them) from being reachable, through
any binding path, from path-matched files outside an allowlist.

Why: some capabilities need a single trusted entry point — a compiler wrapper that
normalizes `typescript` compiler-API usage, or a cache module that owns the only
call site allowed to invalidate broadly. Restricting a named import (or a module's
default export) to owner files keeps that policy enforceable by static analysis
instead of by convention, and catches the capability leaking back in through an
alias, a re-export, `require`, or a dynamic `import()` — not just a direct static
import.

Example: a configured owner path uses the banned export directly.

```ts
// src/internal/typescript-compiler.ts
import { createProgram } from "typescript";

export function compile(rootFiles: string[]) {
  return createProgram(rootFiles, {});
}
```

Counterexample: a checked application file imports the same export directly,
including through an alias.

```ts
// src/app/build.ts
import { createProgram as buildProgram } from "typescript";

export function build(rootFiles: string[]) {
  return buildProgram(rootFiles, {});
}
```

Fix: move the call into the configured owner path and call that wrapper from the
application file.

```ts
// src/app/build.ts
import { compile } from "../internal/typescript-compiler";

export function build(rootFiles: string[]) {
  return compile(rootFiles);
}
```

Options: `checkedPathPatterns`, `allowedPathPatterns`, and `bannedImports`.
`checkedPathPatterns` opts files into the rule; when it is missing or empty, the
rule reports nothing. `allowedPathPatterns` is optional and exempts owner paths
where the banned imports are permitted. `bannedImports` is a list of
`{ module: string, names: string[] }` entries naming the banned exports per
module; the reserved name `"default"` bans the module's default export when it
is used as a directly callable value (and, for CommonJS interop, as a `.default`
member access) — use it alongside named entries like `"invalidateAll"` to ban
both a rate limiter's default export and a specific named method in one entry.
