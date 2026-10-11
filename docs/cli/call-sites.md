# `no-mistakes call-sites`

List every call site of an exported function, with coarse argument shapes.

```sh
no-mistakes call-sites src/api.mts handler --format json
```

Use this to see how a function is actually called before changing its signature.
The query prepares one project-wide call graph and reports invocations whose
resolved callable identity matches the export. Each call site reports `file`,
`line`, enclosing `caller` (when determinable), `argCount`, `hasSpread`, and a
per-argument `args` shape.

Argument shapes are coarse syntactic tags — `string`, `number`, `boolean`,
`null`, `identifier`, `object`, `array`, `arrow`, `call`, `spread`, or `other` —
with no type inference. Named, star, and import-then-export barrels are followed.
Named/default imports, namespace calls (`ns.handler()`), static computed members
(`ns["handler"]()`), and statically resolvable lexical aliases
(`const h = handler; h()`) use the canonical call resolver. Renamed exports use
their local callable identity. A nested binding that shadows an import is
excluded, as are type-only imports.

Dynamic callee selection and aliases invalidated by reassignment may remain
unresolved. Files with parser failures contribute no call sites. Use `rg` on
the returned files when exact call text matters.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `callSites(options)`.

The reverse scan adds `partial` and `incompleteFiles` to structured output.
Each incomplete file reports a root-relative `file`, a `reason`
(`operational-error`, `parse-error`, `missing-facts`, or `missing-symbols`), and a
`message`. The list covers the prepared TS/JS candidate inventory: a failed file
may contain an import that cannot be recovered, so its relevance can be unknown.
Recovered imports may still contribute results when a parser diagnostic exists.
`partial: false` means no recorded collection gaps; it does not guarantee that
all runtime references or unsupported syntax were resolved. When `partial` is
true, treat empty lists and zero counts as known results, not proof of absence.
Human output prints the same warnings. `--format paths` retains the path list
without diagnostics; use JSON before making decisions that depend on absence.
