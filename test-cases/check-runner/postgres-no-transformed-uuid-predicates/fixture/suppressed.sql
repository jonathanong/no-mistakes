-- no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates
SELECT id FROM users WHERE id::text = $1;
SELECT id FROM users WHERE CAST(id AS text) = $1; -- no-mistakes-disable-line postgres-no-transformed-uuid-predicates
-- The unsuppressed control ensures the rule still inspects this source.
SELECT id FROM users WHERE uuid_extract_timestamp(id) > $1;
