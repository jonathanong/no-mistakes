-- lock-reason: prevents: double refund; writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

/* lock-reason: prevents: duplicate job claim; writer: concurrent job runner */ SELECT id FROM jobs WHERE id = $1 FOR NO KEY UPDATE OF jobs NOWAIT;

-- lock-reason: prevents: stale balance read; writer: ledger settlement job
SELECT id
FROM accounts
WHERE id = $1
FOR SHARE SKIP LOCKED;

SELECT id
FROM orders
WHERE id = $1
-- lock-reason: prevents: parent deletion during item insert; writer: order cleanup task
FOR KEY SHARE;

-- lock-reason: prevents: concurrent schema swap
-- writer: nightly backfill worker
LOCK TABLE jobs IN SHARE MODE NOWAIT;

-- lock-reason: prevents: overlapping payout batch; writer: scheduled payout cron
SELECT pg_advisory_lock(42), pg_try_advisory_xact_lock(43);

-- lock-reason: prevents: concurrent ledger close; writer: closing job retry
SELECT acquire_ledger_lock(1);

-- lock-reason: prevents: double debit; writer: payment retry worker
-- no-mistakes: deadlock-safe -- ordered by account_id
SELECT id FROM accounts WHERE id = $1 FOR UPDATE;

SELECT id FROM jobs WHERE id = $1;
