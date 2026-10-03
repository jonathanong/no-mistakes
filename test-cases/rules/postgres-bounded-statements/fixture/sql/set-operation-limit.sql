-- An outer LIMIT does not bound the work of non-streaming input arms.
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT 1;
SELECT id FROM accounts WHERE id = $1 INTERSECT SELECT account_id FROM orders LIMIT 1;
SELECT id FROM accounts WHERE id = $1 UNION SELECT account_id FROM orders LIMIT 1;
-- UNION ALL can stream rows until the cap is reached.
SELECT id FROM accounts UNION ALL SELECT account_id FROM orders LIMIT 1;
-- Both arms independently bounded remain safe.
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders WHERE id = $2 LIMIT 1;
-- A streaming outer arm does not erase non-streaming work in its nested arm.
SELECT id FROM accounts WHERE id = $1 UNION ALL (SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders) LIMIT 1;
