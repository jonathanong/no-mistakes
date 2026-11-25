---
name: queue-rules
description: "Load before writing or reviewing background work: a queue producer or worker (BullMQ, pg-boss, SQS), an enqueue call, a cron or nightly job, a sweep, reconciler, cleanup, backfill, or any loop that reads rows on a schedule. Lists the defects to avoid and the patterns that need a stated reason: producers, workers, leases, dedup, bounded iteration."
---

# Queue and background-job rules

MUST NOT is a defect. SHOULD NOT needs a stated, measured reason.

## Producers and workers

- MUST NOT drop the promise of an enqueue or schedule call; `await` it, or mark deliberate fire-and-forget with `void` and a rejection handler.
- MUST NOT enqueue inside a transaction that may roll back; enqueue after commit. Exception: when the enqueue is the only durable handoff (a consumer whose idempotency row is its only record), enqueue inside the transaction so a failed enqueue rolls the row back; the enqueue must tolerate collisions, and the worker must be idempotent and treat a missing row as a retry.
- MUST NOT rename a job or change its payload incompatibly in one step while jobs of the old shape can exist (queued, delayed, retrying); ship worker support for both shapes, then switch producers, then drop the old once no old-shape jobs remain queued, delayed, or retrying.
- MUST NOT assume a unique job id prevents duplicate runs; it only dedupes enqueues. Make the work idempotent.
- SHOULD NOT key dedup ids by entity alone when distinct events must each run. A retained job still holds its id.
- MUST NOT size a global per-key cap as if it were per-process worker concurrency.
- MUST NOT rely solely on per-worker concurrency or a schedule interval to prevent unsafe overlap between runs of a job. Separate workers, retries of stalled runs, and manual triggers can run concurrently. Make simultaneous runs safe or enforce exclusion across all entry points.
- MUST NOT ship a job whose only failure recovery is the dead-letter queue; add a reconciler or document the accepted loss.
- MUST NOT put secrets or personal data in payloads; pass ids.
- SHOULD NOT put slow external work on a latency-sensitive request path when the caller can receive its result asynchronously.
- MUST NOT treat a dedup id collision as a failure. If finished or failed jobs keep their ids, check that job's state before treating the work as queued.
- SHOULD NOT put mutable state in a payload; pass ids and re-read when the job runs. Facts fixed when the event happened, such as the price charged, belong in the payload.
- SHOULD NOT sleep inside a job; defer it through the queue.
- SHOULD NOT ship bulk work without sizing it against production volume.
- SHOULD NOT keep completed or failed jobs without a bound; set a count or age limit, and keep the durable record of the work in the database.
- MUST NOT set `ttl` on debounce-mode `deduplication` (`{ mode: "debounce", ttl }`); debounce ignores `ttl` (only throttle mode reads it). Enforced by `queues check` (`debounce-ttl`).
- MUST NOT throw a rate-limit error with no delay duration when the queue error carries the provider's `Retry-After` cooldown; pass the positive delay. Enforced by the opt-in `queue-rate-limit-error-delay` rule.

## Leases

- MUST NOT renew, complete, or fail leased work without matching your lease token; zero rows means you lost it.
- SHOULD NOT release leased work without verifying ownership when another worker can acquire it after expiry. Use an atomic ownership check and release.
- SHOULD NOT enter a lease-guarded section after failing to acquire its lease.

## Sweeps and bounded iteration

- SHOULD NOT run a scheduled sweep over a growing source without a finite bound on rows, batches, or time.
- MUST NOT treat a cursor or stream as a work bound; it bounds memory only.
- MUST NOT restart external listings from page one each run; carry the continuation token and stop on a repeat.
- MUST NOT let a delayed repair overwrite a newer value; recompute from current source state or compare an existing timestamp.
- MUST NOT resume from a time watermark that can exclude late-visible rows. Use a commit-ordered cursor or account for late visibility when timestamps can be assigned before commit.
- MUST NOT clear a "needs work" flag unconditionally when a run ends; compare-and-clear against the version you read, or a change that arrived mid-run is lost.
- MUST NOT self-requeue without a no-progress guard.
- SHOULD NOT leave work stranded in a non-terminal state when the event it waits for can be lost.
- SHOULD NOT select work without a stable order when fairness or repeatability matters.
- SHOULD NOT let one sweep's failure or backlog stop an independent sweep from making progress.
- SHOULD NOT let one sweep's mutable state (a tally, budget, or cursor map) decide another sweep's work. See [bounded-iteration](https://github.com/jongleberry/ratchet-stack/blob/main/plugins/ratchet-backend/skills/bounded-iteration/SKILL.md).
