WITH x AS (SELECT created_at FROM invoices)
SELECT 1 FROM orders JOIN x ON orders.id = x.id WHERE created_at > $1;
SELECT 1 FROM orders JOIN (SELECT created_at FROM invoices) d ON orders.id = d.id WHERE created_at > $1;
