# Bounded parse-cache retention

Batch fact collection gives each Rayon file task an owned parse cache. Returning
from the task releases its OXC programs and restores any cache belonging to an
outer task on that worker. All fact variants for the file share that task's
cache, including the historical parser modes used by symbols.

Runner evaluation can parse configuration helpers before batch collection.
Those programs remain on the preparing thread, where all requested variants
are projected from the existing AST. They cannot cross threads, and an owned
worker cache must not force a second parse of these helpers.

Evicting a file removes its interned path and its Standard, TypeScript fallback,
and legacy-symbol entries with three hash-map removals. Its cost is independent
of the number of other cached programs. Other paths and parser failures retain
their request identity.

Saved regressions in `fixtures/check-facts/bounded-session-memory` exercise a
single-worker nested batch and path eviction. The nested batch previously left
two programs in the outer cache instead of one; eviction previously retained the
removed path in the interner. Both tests also protect the unaffected outer path.
The runner-helper regression separately asserts one physical parse per helper.

Total peak RSS also includes retained facts, source snapshots, and evaluator
state. Source snapshots retain one request-wide read identity, including errors.
Any future source release needs complete downstream source demand at the request
boundary; repeated `Arc<str>` references already share the text allocation.
