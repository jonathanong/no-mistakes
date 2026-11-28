SELECT id FROM jobs WHERE id = $1 FOR UPDATE NOWAIT;
SELECT id FROM jobs WHERE id = $1 FOR NO KEY UPDATE SKIP LOCKED;
SELECT id FROM jobs WHERE id = $1 FOR SHARE;
SELECT id FROM jobs WHERE id = $1 FOR KEY SHARE;
LOCK TABLE jobs IN ACCESS EXCLUSIVE MODE;
SELECT pg_advisory_lock(42);
SELECT pg_try_advisory_xact_lock(43);

-- no-mistakes: deadlock-safe -- ordered by account_id
SELECT id FROM accounts WHERE id = $1 FOR UPDATE;

-- lock-reason: writer: refund worker retry
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- lock-reason: prevents: double refund
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- lock-reason: needed for safety
SELECT id FROM refunds WHERE id = $1 FOR UPDATE;

-- lock-reason: prevents: stale comment; writer: someone else

SELECT id FROM orders WHERE id = $1 FOR UPDATE;

SELECT '-- lock-reason: prevents: fake; writer: fake' FROM orders WHERE id = $1 FOR UPDATE;

/* lock-reason: prevents: a; writer: b */ SELECT id FROM a FOR UPDATE; SELECT id FROM b FOR UPDATE;
