-- no-mistakes-disable-next-line postgres-lock-reason: intentional migration lock
SELECT id FROM jobs WHERE id = $1 FOR UPDATE;

SELECT id FROM accounts WHERE id = $1 FOR SHARE; -- no-mistakes-disable-line postgres-lock-reason: intentional read lock
