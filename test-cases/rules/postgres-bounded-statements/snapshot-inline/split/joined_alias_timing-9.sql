DELETE FROM accounts AS j WHERE id IN (SELECT j.account_id FROM (accounts a JOIN orders o ON (SELECT j.id) = o.account_id) AS j LIMIT 1);
