DELETE FROM orders WHERE id IN (SELECT id FROM orders);
