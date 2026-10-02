SELECT orders.id FROM orders
JOIN tags ON created_at > $1
JOIN invoices ON invoices.id = orders.id;
