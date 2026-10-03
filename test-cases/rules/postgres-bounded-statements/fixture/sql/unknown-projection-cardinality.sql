-- Unknown functions can expand a bind input into database-backed rows.
UPDATE accounts a SET name = 'x' FROM (SELECT custom_srf($1) AS id) ids WHERE a.id = ids.id;
-- Schema-qualified calls have the same unknown cardinality.
UPDATE accounts a SET name = 'x' FROM (SELECT public.custom_srf($1) AS id) ids WHERE a.id = ids.id;
-- A user-schema lookalike of a scalar builtin remains unknown.
UPDATE accounts a SET name = 'x' FROM (SELECT public.lower($1) AS id) ids WHERE a.id = ids.id;
-- A user-schema lookalike of an aggregate also remains unknown.
UPDATE accounts a SET name = 'x' FROM (SELECT public.count($1) AS id) ids WHERE a.id = ids.id;
-- Unknown projection calls can expand an otherwise scalar aggregate group.
UPDATE accounts a SET name = 'x' FROM (SELECT custom_srf(md5(count(*)::text)::uuid) AS id FROM orders) ids WHERE a.id = ids.id;
-- Known scalar builtins keep one output row and cannot introduce a row source.
UPDATE accounts a SET name = 'x' FROM (SELECT lower($1) AS email) ids WHERE a.email = ids.email;
UPDATE accounts a SET name = 'x' FROM (SELECT pg_catalog.lower($1) AS email) ids WHERE a.email = ids.email;
-- Aggregates retain their single-group cardinality when not expanded by unknown calls.
UPDATE accounts a SET name = 'x' FROM (SELECT count(*)::text AS email FROM orders) ids WHERE a.email = ids.email;
UPDATE accounts a SET name = 'x' FROM (SELECT lower(max(name)) AS email FROM orders) ids WHERE a.email = ids.email;
-- Known caller-sized SRFs and explicit LIMIT caps remain valid bounds.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest($1::uuid[]) AS id) ids WHERE a.id = ids.id;
UPDATE accounts a SET name = 'x' FROM (SELECT custom_srf($1) AS id LIMIT 1) ids WHERE a.id = ids.id;
-- A trusted scalar wrapper does not hide an unknown nested set-returning call.
UPDATE accounts a SET name = 'x' FROM (SELECT lower(custom_srf($1)::text) AS email) ids WHERE a.email = ids.email;
-- COALESCE retains scalar cardinality; PostgreSQL rejects an SRF nested in this form.
UPDATE accounts a SET name = 'x' FROM (SELECT coalesce($1, $2) AS id) ids WHERE a.id = ids.id;
