SELECT id FROM orders JOIN invoices ON orders.id = invoices.id WHERE created_at > $1;
