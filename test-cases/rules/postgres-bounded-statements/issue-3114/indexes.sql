SELECT receipt_id FROM receipts WHERE receipt_id = $1;
SELECT slug FROM partial_keys WHERE slug = 'literal' AND deleted_at IS NULL;
SELECT slug FROM expr_keys WHERE LOWER(slug) = LOWER($1);
SELECT username FROM usernames WHERE LOWER(username) = LOWER($1) AND kind = 'active';
SELECT slug FROM partial_keys WHERE slug = 'literal';
SELECT username FROM usernames WHERE LOWER(username) = LOWER($1);
SELECT slug FROM expr_keys WHERE LENGTH(slug) = $1;
