-- The later j child cannot shadow the outer write alias in an earlier ON.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
