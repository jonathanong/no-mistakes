DELETE FROM accounts AS j WHERE id IN (SELECT j2.child_id FROM (accounts a JOIN orders o ON true CROSS JOIN LATERAL unnest(ARRAY[j.id]) u(child_id)) AS j2 LIMIT 1);
