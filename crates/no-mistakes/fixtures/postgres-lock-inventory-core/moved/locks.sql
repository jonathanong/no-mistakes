-- The account lock moved above the job locks.
SELECT id FROM accounts FOR KEY SHARE;
select id from jobs for /* wait */ update;
SELECT reserveJob (7);
SELECT id
FROM jobs FOR UPDATE;
SELECT "reserveJOB"(7);
