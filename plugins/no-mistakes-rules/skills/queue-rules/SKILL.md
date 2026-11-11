---
name: queue-rules
description: "MUST NOT / SHOULD NOT rules for background work: queue producers and workers, scheduled jobs, sweeps, reconcilers, backfills, leases, and dedup. Use when writing or reviewing a job, a worker, an enqueue call, or any loop that reads rows on a schedule."
---

# Queue and background-job rules

MUST NOT is a defect. SHOULD NOT needs a stated, measured reason.

## Producers and workers

- MUST NOT drop the promise of an enqueue or schedule call; `await` it, or mark deliberate fire-and-forget with `void` and a rejection handler.
- MUST NOT enqueue inside a transaction that may roll back; enqueue after commit. Exception: when the enqueue is the only durable handoff (a consumer whose idempotency row is its only record), enqueue inside the transaction so a failed enqueue rolls the row back; the enqueue must tolerate collisions, and the worker must be idempotent and treat a missing row as a retry.
- MUST NOT rename a job or change its payload in one step; ship worker support for both shapes, then switch producers, then drop the old.
- MUST NOT assume a unique job id prevents duplicate runs; it only dedupes enqueues. Make the work idempotent.
- MUST NOT key dedup ids by entity alone; include the triggering event, because retained jobs still hold the id.
- MUST NOT put shared-queue jobs in no ordering lane or with implicit concurrency; one family starves the rest.
- MUST NOT expect priority to reorder work inside one FIFO lane; give urgent work its own lane.
- MUST NOT size a global per-key cap as if it were per-process worker concurrency.
- MUST NOT put high-volume work or dispatchers in the same lane as ordinary tasks.
- MUST NOT ship a job whose only failure recovery is the dead-letter queue; add a reconciler or document the accepted loss.
- MUST NOT put secrets or personal data in payloads; pass ids.
- MUST NOT make producer code import worker code; producers enqueue, workers execute.
- MUST NOT do slow external work on a request path or in an event-forwarding function; enqueue it.
- MUST NOT put mutable state in a payload; pass ids and re-read when the job runs.
- MUST NOT treat a dedup id collision as a failure; the work is already queued.
- MUST NOT sleep inside a job; defer it through the queue.
- MUST NOT do the work inside a message consumer; enqueue a job, then ack.
- MUST NOT trust a framework's retry counter without checking that no middleware resets it.
- MUST NOT expect a code deploy to change concurrency or prefetch settings read at worker start; restart the workers.
- MUST NOT ship bulk work without sizing it against production volume.
- SHOULD NOT put business logic in processors; call the service that owns the write.
- SHOULD NOT add an outbox or lease table by default; use job id = row id, enqueue after commit, and a bounded re-enqueue sweep.

## Leases

- MUST NOT renew, complete, or fail leased work without matching your lease token; zero rows means you lost it.
- MUST NOT set a lease TTL below the job's lock duration, or release a lease with a plain delete; compare-and-delete by token.
- MUST NOT hold a lease across a deferral that never returns; release first.
- MUST NOT let a lease-contention path fall through into the guarded work; defer this job with bounded, jittered delay.

## Sweeps and bounded iteration

- MUST NOT loop until empty or default a cap to infinity; cap rows, batches, or time per run, persist position, report remaining work.
- MUST NOT treat a cursor or stream as a work bound; it bounds memory only.
- MUST NOT fan out with unbounded `Promise.all`; bound both the input set and concurrency.
- MUST NOT scan every entity to find work; read work-item rows through an index-served predicate.
- MUST NOT restart external listings from page one each run; carry the continuation token and stop on a repeat.
- MUST NOT let a delayed repair overwrite a newer value; compare a monotonic version.
- MUST NOT resume a time window without overlap.
- MUST NOT hardcode batch size, page size, or window as literals; bind validated config.
- MUST NOT clear a "needs work" flag unconditionally when a run ends; compare-and-clear against the version you read, or a change that arrived mid-run is lost.
- MUST NOT self-requeue without a no-progress guard.
- MUST NOT leave a non-terminal state without a timeout exit, or pick work without an `ORDER BY`.
- SHOULD NOT combine unrelated triggers in one sweep with `OR`; one dispatcher per signal, same idempotent job.
- SHOULD NOT add a new sweeper for a repair an existing reconciler can make idempotently.
