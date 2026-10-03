-- One pinned source row can store arbitrarily many account ids.
UPDATE accounts a SET name = 'x' FROM orders o
WHERE o.id = $1 AND a.id = ANY(o.account_ids);
UPDATE accounts a SET name = 'x' FROM orders o
WHERE o.id = $1 AND a.id = ANY(coalesce(o.account_ids, ARRAY[$2]));
DELETE FROM accounts WHERE id = ANY($1::uuid[]);
DELETE FROM accounts WHERE id = ANY(ARRAY[$1, $2]);
UPDATE accounts a SET name = 'x' FROM orders o
WHERE o.id = $1 AND a.id = o.account_id;
