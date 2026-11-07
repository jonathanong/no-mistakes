SELECT id FROM accounts WHERE COALESCE(token, id) IS NOT NULL;
SELECT id FROM accounts WHERE COALESCE(token, id) IS NULL;
SELECT id FROM accounts WHERE (id::text) IS NOT NULL;
SELECT id FROM accounts WHERE (id::text) IS NULL;
