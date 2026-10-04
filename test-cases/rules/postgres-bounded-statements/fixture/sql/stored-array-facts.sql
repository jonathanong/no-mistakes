-- A uniquely pinned array owner still supplies an arbitrarily large key set.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(o.account_ids);
UPDATE accounts a SET name = 'x' FROM orders o WHERE a.id = ANY(o.account_ids);
DELETE FROM accounts a WHERE a.id = ANY(a.account_ids);
-- Caller arrays and finite constructors retain their existing finite proof.
DELETE FROM accounts WHERE id = ANY($1::uuid[]);
DELETE FROM accounts WHERE id = ANY(ARRAY[$1::uuid, $2::uuid]);
DELETE FROM accounts WHERE id = $1;
-- A stored array hidden behind a cast remains a stored source.
UPDATE accounts a SET name = 'x' FROM orders o WHERE o.id = $1 AND a.id = ANY(o.account_ids::uuid[]);
-- Bare reads inside an array expression retain catalog-deferred ownership metadata.
DELETE FROM accounts WHERE id = ANY(coalesce((SELECT account_ids FROM orders LIMIT 1), $1));
