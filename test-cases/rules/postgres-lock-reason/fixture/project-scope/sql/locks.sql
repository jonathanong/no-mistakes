-- lock-reason: prevents: duplicate job claim; writer: worker retry
SELECT id FROM jobs FOR UPDATE;

SELECT id FROM accounts FOR SHARE;
