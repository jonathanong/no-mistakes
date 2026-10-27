DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM ((accounts a JOIN orders o ON j.id = o.account_id) AS j JOIN (SELECT 1 AS marker) extra ON true) AS k LIMIT 1);
