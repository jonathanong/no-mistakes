DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a RIGHT OUTER JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
