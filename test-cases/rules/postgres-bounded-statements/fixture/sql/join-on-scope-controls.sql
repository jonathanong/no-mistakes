-- Every PostgreSQL ON join spelling uses its own earlier-child namespace.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a INNER JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a LEFT JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a LEFT OUTER JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a RIGHT JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a RIGHT OUTER JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a FULL OUTER JOIN orders o ON a.id = o.account_id AND j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
-- Non-ON function arguments still use the surrounding group capture.
DELETE FROM accounts AS j WHERE id IN (SELECT j2.child_id FROM (accounts a JOIN orders o ON true CROSS JOIN LATERAL unnest(ARRAY[j.id]) u(child_id)) AS j2 LIMIT 1);
-- A bare physical child column in a function input is still local.
DELETE FROM accounts AS j WHERE id IN (SELECT j2.child_id FROM (accounts a JOIN orders o ON true CROSS JOIN LATERAL unnest(ARRAY[account_id]) u(child_id)) AS j2 LIMIT 1);
