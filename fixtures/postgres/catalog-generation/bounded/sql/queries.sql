SELECT id FROM invoices WHERE paid_at IS NULL;
UPDATE exports SET s3_key = NULL WHERE expires_at < now();
SELECT name FROM accounts WHERE email = $1;
SELECT id FROM orders WHERE slug = $1;
SELECT 1 FROM tokens WHERE token = $1;
WITH c AS (SELECT id FROM exports WHERE expires_at < now() ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED)
  UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id;
SELECT code FROM currencies;
SELECT count(*) FROM orders WHERE account_id = $1;
