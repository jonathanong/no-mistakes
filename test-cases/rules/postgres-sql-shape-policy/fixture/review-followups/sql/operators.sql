SELECT id FROM accounts
WHERE (SELECT COUNT(*) FROM orders) != 0;
SELECT id FROM accounts
WHERE (SELECT COUNT(*) FROM orders) <> 0;
