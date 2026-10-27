DELETE FROM accounts AS j WHERE id IN (SELECT j.child_id FROM (accounts a JOIN LATERAL (SELECT j.id AS child_id) b ON true) AS j LIMIT 1);
