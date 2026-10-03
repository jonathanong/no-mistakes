-- no-mistakes-disable-next-line postgres-bounded-statements
SELECT id FROM invoices;
SELECT id FROM invoices; -- no-mistakes-disable-line postgres-bounded-statements
SELECT id FROM invoices;
-- A directive at a statement start covers each later relation in that statement.
-- no-mistakes-disable-next-line postgres-bounded-statements
SELECT id
FROM invoices;
SELECT id -- no-mistakes-disable-line postgres-bounded-statements
FROM invoices;
-- no-mistakes-disable-next-line postgres-bounded-statements
UPDATE
exports SET s3_key = NULL;
UPDATE -- no-mistakes-disable-line postgres-bounded-statements
exports SET s3_key = NULL;
-- no-mistakes-disable-next-line postgres-bounded-statements
DELETE
FROM sessions;
DELETE -- no-mistakes-disable-line postgres-bounded-statements
FROM sessions;
SELECT id
FROM invoices; -- no-mistakes-disable-line postgres-bounded-statements
