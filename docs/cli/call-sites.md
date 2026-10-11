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
with no type inference. Named and star re-export barrels are followed
transparently, including import-then-re-export chains; barrel files themselves
are not scanned for calls. The defining file is scanned under the export's
local binding, so calls to a renamed export's implementation
(`function impl(){}; export { impl as handler }`) are included. Only direct
identifier calls (`handler(...)`) match; namespace member calls (`ns.handler()`),
indirect aliases (`const h = handler; h()`), and a local binding that shadows
the import inside a nested scope are not resolved. A type-only import
(`import type { fn }`) is treated like any other binding, so a same-named value
call in that file may be reported. A file that fails to parse (e.g. mid-edit)
contributes no call sites rather than failing the query, so results can be
incomplete. Use `rg` on the returned files when exact call text matters.

Key options: `--root`, `--tsconfig`, `--format`, and `--json`.

Node API: `callSites(options)`.
