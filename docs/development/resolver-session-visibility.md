# Request-owned resolver visibility

Resolver visibility is interned within `AnalysisSession`. A content fingerprint
selects a bucket; full path membership decides whether two projections are
identical. The fingerprint never determines a known import target by itself.
Retained identities make selected resolver keys independent of repository size.
Standalone resolvers retain their complete path keys.

Each exact projection prepares normalized paths and canonical aliases once.
Canonicalization successes and failures are memoized per lexical path across
projections. Equal projections from separate domains retain one lookup and
share positive and negative import results. Narrower integration catalogs and
broader runner catalogs remain distinct. Queue compatibility retains its own
root and resolution policy in the cache identity.

Prepared graph classifications use this same session import table. Legacy
lookups with custom alias remapping retain their explicit raw projection;
attaching that projection never clears an already warmed session table.
Identity canonicalizations retain no duplicate canonical path text, and a
request that never prepares resolver visibility allocates no visibility registry.

Runner configuration preparation precedes graph-session preparation. Its
existing prepared owner retains one frozen visibility projection and resolved
imports across configuration files and clones. This adds no competing session,
source store, discovery, or TS fact pass. Importer/config selection is immutable
within this owner.

Verbose diagnostics add these request work counters:

- `resolver.visible_normalizations`: distinct visibility projections normalized.
- `resolver.visible_canonicalize_calls`: actual path canonicalization attempts,
  including failures; overlapping projections reuse those attempts.
- `resolver.scope_constructions`: exact session resolver caches first created,
  including distinct queue policies and roots.

The existing `resolver.computations` counter measures actual import resolution.
Counters are absent when diagnostics are disabled. A request with genuinely
different projections may normalize more than one projection; equal domains
must normalize only once, and each lexical path is canonicalized once.

## Validation and measurements

Fixture-backed tests cover equal but separately allocated sets, concurrent
construction, unresolved imports, additive narrower/broader scopes, hash bucket
collisions, canonical aliases, failed canonicalization, queue policy/root
separation, and runner configuration reuse. Existing resolver/domain fixtures
protect public findings and catalog selection.

Matched measurement results and source provenance are recorded for this change
before handoff. The saved consuming/control configurations are in
`fixtures/performance/session-visible-resolver/`. They use the same physical
repository paths and explicitly limit checks to the Node package. Existing
`core-analysis` and `graph-gates` fixtures retain JSX and configured include
payloads. No issue estimate is a measured speed claim.
