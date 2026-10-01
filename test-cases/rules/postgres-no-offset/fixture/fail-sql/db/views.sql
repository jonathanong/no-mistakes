CREATE VIEW view_recent_orders AS
SELECT * FROM (SELECT id, account_id FROM orders WHERE id > '0190' OFFSET 0) o JOIN accounts a ON a.id = o.account_id;
SELECT id FROM orders ORDER BY id LIMIT 20 OFFSET 40;
