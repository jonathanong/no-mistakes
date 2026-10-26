SELECT 1;
-- Malformed INSERTs are relevant; the other broken SELECT is not replay-safety evidence.
INSERT INTO orders(id) VALUES (;
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
-- no-mistakes-disable-next-line postgres-idempotent-insert
INSERT INTO orders(id) VALUES (;
INSERT INTO orders(id) VALUES ($1) ON CONFLICT DO NOTHING;
