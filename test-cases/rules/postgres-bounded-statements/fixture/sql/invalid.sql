SELECT id FROM invoices WHERE paid_at IS NULL;
UPDATE exports SET s3_key = NULL WHERE expires_at < now();
DELETE FROM sessions WHERE expires_at < now();
