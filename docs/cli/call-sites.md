# `no-mistakes call-sites`

List every call site of an exported function, with coarse argument shapes.

```sh
no-mistakes call-sites src/api.mts handler --format json
```

Use this to see how a function is actually called before changing its signature.
The query is scoped to files that import the export (plus the defining file), so
it stays fast. Each call site reports the `file`, `line`, enclosing `caller`
(when determinable), `argCount`, `hasSpread`, and a per-argument `args` shape.

Argument shapes are coarse syntactic tags — `string`, `number`, `boolean`,
`null`, `identifier`, `object`, `array`, `arrow`, `call`, `spread`, or `other` —
with no type inference. Only direct identifier calls (`handler(...)`) match;
namespace member calls (`ns.handler()`) and indirect aliases
(`const h = handler; h()`) are not resolved. Use `rg` on the returned files when
exact call text matters.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `callSites(options)`.
