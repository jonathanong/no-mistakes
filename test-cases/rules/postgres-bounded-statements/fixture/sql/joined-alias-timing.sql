-- The joined alias is unavailable inside its own ON; j.id is the outer write row.
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON j.id = o.account_id) AS j LIMIT 1);
-- An actual child alias is visible in ON and preserves an independent capped key set.
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON a.id = o.account_id) AS j LIMIT 1);
-- LATERAL cannot read the alias that will be assigned after its joined group finishes.
DELETE FROM accounts AS j WHERE id IN (SELECT j.child_id FROM (accounts a JOIN LATERAL (SELECT j.id AS child_id) b ON true) AS j LIMIT 1);
-- LATERAL still sees the already visited child a.
DELETE FROM accounts AS j WHERE id IN (SELECT j.child_id FROM (accounts a JOIN LATERAL (SELECT a.id AS child_id) b ON true) AS j LIMIT 1);
-- An enclosing join alias cannot consume an outward read from a nested join's ON.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM ((accounts a JOIN orders o ON j.id = o.account_id) AS j JOIN (SELECT 1 AS marker) extra ON true) AS k LIMIT 1);
-- A later child named j is unavailable to a preceding LATERAL query.
DELETE FROM accounts AS j WHERE id IN (SELECT k.child_id FROM (orders o JOIN LATERAL (SELECT j.id AS child_id) b ON true JOIN accounts j ON true) AS k LIMIT 1);
-- The same timing rule applies to whole-row references inside ON.
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON (j).id = o.account_id) AS j LIMIT 1);
-- A child whole-row reference inside ON remains local.
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON (a).id = o.account_id) AS j LIMIT 1);
-- A nested scalar query inside ON must preserve the outer alias reference too.
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON (SELECT j.id) = o.account_id) AS j LIMIT 1);
