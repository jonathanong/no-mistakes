DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON a.id = o.account_id) AS j LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT j.child_id FROM (accounts a JOIN LATERAL (SELECT a.id AS child_id) b ON true) AS j LIMIT 1);
DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON (a).id = o.account_id) AS j LIMIT 1);
