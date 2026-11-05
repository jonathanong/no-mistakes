# Numeric ternary SQL consumers

`numeric.ts` and `string.ts` differ only in ternary literal types. Their SQL
alternatives are identical. Each configuration enables the same seventeen
embedded SQL consumers with an identical executor profile.

The concrete statements exercise bounded reads, explicit columns, required
predicates, generated timestamp rewrites and writes, transformed UUIDs,
non-idempotent INSERTs, conflict ordering, test mutations, OFFSET, lock ordering,
literal LIMIT shape policy, annotation heads, and JSON relation identifiers.
The final two trusted-tag calls embed a raw numeric-precision template beside
an environment or mutable-registry relation hole. Recovering the inner numeric
ternary makes the outer SQL parseable, exposing both provenance findings. Numeric
type precision requires a literal; an unrecovered placeholder fails parsing.
Literal-only statements remain negative controls for identifier provenance.

The preceding untagged locking statement is deliberately invalid SQL with a recovered locking clause.
Lock ordering must retain its unparseable finding under both policies. Other
consumers retain independently decidable facts while ignore suppresses unknowns.
The fail configuration uses the annotation rule's legacy `report` spelling.

Snapshot cases run at one and two jobs. The companion assertion compares concrete
numeric/string finding identities and requires every concrete consumer to report.
