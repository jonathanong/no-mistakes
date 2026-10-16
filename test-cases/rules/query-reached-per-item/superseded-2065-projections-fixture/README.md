# Preserved provenance cases

These original TypeScript inputs come byte-for-byte from superseded PR #2065 at
`d0140d39c8d5b5540f6e47fd299b236ce1832a3d`. Their original comments describe that
candidate's intent. `expected-lines.json` describes the merged implementation's
policy instead; the retired mixed-alternative policy is not adopted.

The selected controls cover direct and finite indexed imported types, explicit
namespace/barrel identities, directly typed callbacks, returned executors, known
non-nullish callables, bound callables, and copied-member acquisition.

The negative controls have separate reasons:

- Local classes, forward aliases, generic parameters, and utility imports own
  their lexical type identity. A matching spelling cannot borrow an import.
- An unrelated or unknown nullish alternative prevents selection. A known
  non-nullish callable may discard an unreachable fallback; a type annotation
  alone does not establish this runtime fact.
- Unawaited promises and their cast annotations are not callable executors.
- Recursive indexed descriptors and recursive barrels remain unresolved. The
  finite paths through a recursive object are also characterized as unsupported.
- Untyped relays and indexed callback signatures do not supply contextual
  parameter annotations. Conflicting authored contexts cannot borrow each
  other's type, and reassigned callbacks lose the original binding evidence.
- Empty, missing, numeric, computed, primitive, and generic callable shapes do not
  invent a configured imported executor identity.

The companion import-order fixture configures the custom imported utility itself,
so its early, late, and indexed forms select that identity. Its lexical shadows
remain negative. The plain companion retains one non-nullish report and asserts
one request-scoped graph and parse, without copying the retired implementation's
memoization tests or performance claims.
