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
