CREATE VIEW view_recent_orders AS
WITH o AS MATERIALIZED (SELECT id, account_id FROM orders WHERE id > '0190')
SELECT o.id FROM o JOIN accounts a ON a.id = o.account_id;
