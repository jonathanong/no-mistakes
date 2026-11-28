-- BEGIN; COMMIT; ROLLBACK; (comment must stay inert)
SELECT 'BEGIN; START TRANSACTION; COMMIT; END; ROLLBACK; SAVEPOINT sp;';
-- no-mistakes-disable-next-line postgres-sql-statement-policy
BEGIN;
BEGIN WORK;
START TRANSACTION READ ONLY;
SAVEPOINT before_backfill;
ROLLBACK TO SAVEPOINT before_backfill;
RELEASE SAVEPOINT before_backfill;
COMMIT AND CHAIN;
ROLLBACK TRANSACTION;
END WORK;
