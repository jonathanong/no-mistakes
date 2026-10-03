-- A function result may be an unbounded array read from the database.
UPDATE accounts a SET name = 'x' FROM unnest(get_all_account_ids()) u(id) WHERE a.id = u.id;
DELETE FROM accounts WHERE id IN (SELECT id FROM unnest(pg_catalog.get_all_account_ids()) u(id));
-- A call nested inside a caller-shaped expression is still not caller-sized.
UPDATE accounts a SET name = 'x' FROM unnest(ARRAY[get_account_id()]) u(id) WHERE a.id = u.id;
-- Literal and caller-supplied arrays remain bounded.
UPDATE accounts a SET name = 'x' FROM unnest($1::uuid[]) u(id) WHERE a.id = u.id;
UPDATE accounts a SET name = 'x' FROM unnest(ARRAY[$1, $2]) u(id) WHERE a.id = u.id;
