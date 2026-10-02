UPDATE orders SET updated_at = now();
INSERT INTO orders VALUES (1, now(), 'note');
UPDATE orders SET old_generated = 1;
