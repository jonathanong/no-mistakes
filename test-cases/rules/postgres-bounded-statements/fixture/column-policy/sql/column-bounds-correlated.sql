SELECT target.id FROM accounts target
JOIN LATERAL (SELECT x.id FROM orders x WHERE x.account_id = target.id) child ON true
WHERE target.id = $1;
SELECT target.id FROM accounts target
JOIN LATERAL (SELECT x.id FROM orders x WHERE x.account_id = target.id) child ON true;
SELECT target.id FROM accounts target
JOIN LATERAL (SELECT x.id FROM orders x WHERE x.account_id = missing.id) child ON true
WHERE target.id = $1;
SELECT target.id FROM accounts target
JOIN LATERAL (SELECT x.id FROM orders x WHERE x.account_id = x.id) child ON true
WHERE target.id = $1;
