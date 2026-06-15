# `no-mistakes registry-extension`

Summarize the repeated "register an entry" shape used in a registry file so you
can add a new entry that follows the same pattern.

```sh
no-mistakes registry-extension src/plugins/index.ts --root . --format json
```

Use this before adding an entry to an extension point: it shows what each
existing entry imports (module specifier + symbol) and the verbatim
call/config shape used to register it, plus a best-effort `template`.

## Heuristics and limits

This command is intentionally heuristic. Three detectors run over the file and
the dominant one (by entry count, minimum 2) is reported:

1. **`register-call`** — a callee invoked ≥ 2 times whose argument references an
   imported symbol, e.g. `registry.register(new FooPlugin())` or `app.use(x)`.
2. **`container-array`** / **`container-object`** — a default-exported array or
   object literal whose elements/values are entries
   (`export default [new A(), new B()]`).
3. **side-effect imports** (`import "./plugins/foo"`) are reported as a
   secondary `note`, not as the primary pattern.

Mixed-shape files report the dominant shape and mention the others in `notes`;
nothing is silently dropped. `confidence` is `high` when every entry resolves to
an import, otherwise `medium`. A file with no repeated pattern reports
`patternKind: "none"` with `confidence: "low"` and empty `entries`. Aliased
imports are matched by local name and reported via their imported symbol +
source; dynamic-import registrants (`() => import("./x")`) get
`kind: "dynamic"` with a null `symbol`.

Known heuristic gaps (best-effort): namespace-imported factory calls
(`registry.register(plugins.makeFoo())`) may not resolve to an import; entries
wrapped in a local helper (`registry.register(makeEntry(Foo))`) can misattribute
the registrant; and the generated `template` placeholder is approximate for
namespace-import constructors.

Key options: `--format`, `--json`.

Output shape:

```json
{
  "registryFile": "src/plugins/index.ts",
  "patternKind": "register-call",
  "registrant": "registry.register",
  "confidence": "high",
  "entries": [
    {
      "line": 5,
      "import": { "specifier": "./plugins/foo", "symbol": "FooPlugin", "local": "FooPlugin", "kind": "static" },
      "callShape": "registry.register(new FooPlugin({ id: \"foo\" }))"
    }
  ],
  "template": "registry.register(new <Entry>({ id: \"foo\" }))",
  "notes": []
}
```

Node API: `registryExtension(options)`.
