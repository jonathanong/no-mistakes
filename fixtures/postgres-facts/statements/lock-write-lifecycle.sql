PREPARE ignored(integer) AS UPDATE ignored SET value = 1 WHERE id = $1;
CREATE TEMP TABLE scratch(id integer);
UPDATE accounts SET value = 1 WHERE id = $1;
UPDATE accounts SET value = 2 FROM sources WHERE accounts.id = sources.id;
DELETE FROM accounts WHERE id = $1;
DELETE FROM accounts USING sources WHERE accounts.id = sources.id;
LOCK TABLE accounts, sources IN ACCESS EXCLUSIVE MODE;
COMMIT;
ROLLBACK;
