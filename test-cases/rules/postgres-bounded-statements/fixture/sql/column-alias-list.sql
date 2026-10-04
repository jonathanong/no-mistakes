-- Positional aliases can rename a non-key column to a catalog key name.
SELECT 1 FROM accounts AS a(tenant_id, id) WHERE a.id = $1;
SELECT 1 FROM accounts AS a WHERE a.id = $1;
SELECT 1 FROM accounts AS a(tenant_id, id) JOIN orders o ON a.id = o.account_id WHERE o.id = $1;
SELECT 1 FROM accounts AS a(tenant_id, id) JOIN orders o USING (id) WHERE o.id = $1;
-- Catalog key credit is withheld, but nested pin queries remain in public syntactic facts.
SELECT 1 FROM accounts AS a(tenant_id, id) WHERE a.id IN (SELECT id FROM orders);
