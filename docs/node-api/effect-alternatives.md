# Effect alternatives

The async `effects()` API and `analyzeProject()` effects report use the same
physical-occurrence contract as [the CLI](../cli/effects.md). The exported
`EffectCallAlternative` type names `{ callee: string; category?: string }`.
`EffectCallSite.alternatives` is optional and appears only when multiple distinct
configured matches remain; ordinary singleton JSON stays unchanged. Alternatives
are sorted by callee/category, with the first match also supplying scalar fields.

Each `byCategory` count deduplicates a physical occurrence within that category;
several categories may describe one row. `categories` filters alternatives before
choosing scalar fields and counts, so a read-only request cannot expose a write
scalar/category count. Unknown candidates add no evidence. The returned Promise,
CommonJS/ESM facades and batched reports preserve this same projection.


`check()` and `analyzeProject()` accept the same `forbidden-calls` argument
predicates as the CLI through rule configuration. Both remain asynchronous and
share prepared request facts. `ForbiddenCallArgumentLiteral`,
`ForbiddenCallArgumentPredicate`, `ForbiddenCallArgumentConditions`, and
`ForbiddenCallUnknownArguments` are exported declaration types. See
[bounded argument predicates](../rules/forbidden-calls.md#bounded-argument-predicates)
for the explicit unknown policy and source-analysis limits.

[Node/N-API guide](../node-api.md)
