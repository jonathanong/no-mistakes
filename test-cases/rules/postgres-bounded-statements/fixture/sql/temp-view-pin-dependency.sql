-- DROP CASCADE retires a temporary view that reads a permanent table through a correlated pin.
CREATE TEMP VIEW accounts AS SELECT a.id FROM accounts a WHERE a.id IN (SELECT o.id FROM orders o WHERE o.account_id = a.id);
DROP TABLE orders CASCADE;
SELECT * FROM accounts;
