UPDATE orders SET updated_at = now();
INSERT INTO orders VALUES (1, 'paid', DEFAULT, now());
