DELETE FROM accounts AS j WHERE id IN (SELECT k.child_id FROM (orders o JOIN LATERAL (SELECT j.id AS child_id) b ON true JOIN accounts j ON true) AS k LIMIT 1);
