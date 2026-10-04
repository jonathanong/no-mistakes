-- The later j child cannot shadow the outer write alias in an earlier ON.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
-- A real earlier child reference remains local despite the later shadowing alias.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a JOIN orders o ON a.id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
-- Whole-row and scalar-subquery references retain the earlier ON namespace too.
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a JOIN orders o ON (j).id = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT k.account_id FROM (accounts a JOIN orders o ON (SELECT j.id) = o.account_id JOIN (SELECT 1 AS marker) j ON true) AS k LIMIT 1);
-- Ordinary joined groups use the same per-ON boundary without a surrounding alias.
DELETE FROM accounts AS j WHERE id IN (SELECT o.account_id FROM accounts a JOIN orders o ON j.id = o.account_id JOIN (SELECT 1 AS marker) j ON true LIMIT 1);
