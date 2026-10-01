SELECT id FROM orders JOIN tags ON orders.id = tags.id WHERE created_at > $1;
