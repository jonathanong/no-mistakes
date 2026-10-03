-- A FROM-free derived query can still emit database-sized rows through an SRF.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest((SELECT array_agg(id) FROM accounts)) AS id) ids WHERE a.id = ids.id;
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_account_ids()));
UPDATE accounts a SET name = 'x' FROM (SELECT app.unnest($1::uuid[]) AS id) ids WHERE a.id = ids.id;
-- Caller-sized projections and explicit caps retain their bounds.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest($1::uuid[]) AS id) ids WHERE a.id = ids.id;
DELETE FROM accounts WHERE id IN (SELECT unnest(get_all_account_ids()) LIMIT 1);
UPDATE accounts a SET name = 'x' FROM (SELECT generate_series(1, 10) AS id) ids WHERE a.id = ids.id;
-- Named arguments and recovered interpolation identifiers follow the same input proof.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest(input := get_all_account_ids()) AS id) ids WHERE a.id = ids.id;
UPDATE accounts a SET name = 'x' FROM (SELECT unnest(input := sql_placeholder_1) AS id) ids WHERE a.id = ids.id;
UPDATE accounts a SET name = 'x' FROM (SELECT unnest(ids) AS id FROM orders) ids WHERE a.id = ids.id;
-- The SQL parser accepts these unsupported argument forms; they must never prove a bound.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest(*) AS id) ids WHERE a.id = ids.id;
-- Arrow notation and named FROM functions use the same caller-input proof.
UPDATE accounts a SET name = 'x' FROM (SELECT unnest(input => sql_placeholder_1) AS id) ids WHERE a.id = ids.id;
UPDATE accounts a SET name = 'x' FROM generate_series(start := $1, stop := 10) ids(id) WHERE a.id = ids.id;
