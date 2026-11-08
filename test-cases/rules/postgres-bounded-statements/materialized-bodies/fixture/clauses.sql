SELECT o.id FROM orders o JOIN accounts a ON EXISTS (SELECT id FROM orders) LIMIT 1;
SELECT o.id FROM orders o JOIN accounts a ON EXISTS (SELECT id FROM orders LIMIT 1) LIMIT 1;
SELECT id FROM orders ORDER BY (SELECT id FROM accounts) LIMIT 1;
SELECT id FROM orders ORDER BY (SELECT id FROM accounts LIMIT 1) LIMIT 1;
SELECT (TABLE "Orders"), (TABLE "Orders") FROM (TABLE accounts LIMIT 1) source LIMIT 1;
SELECT (TABLE "Orders" LIMIT 1), (TABLE "Orders" LIMIT 1) FROM (TABLE accounts LIMIT 1) source LIMIT 1;
