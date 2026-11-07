SELECT id FROM accounts WHERE COALESCE(token, id) = $1;
SELECT id FROM accounts WHERE COALESCE(id::text = $1, false) IS NOT NULL;
SELECT id FROM accounts WHERE COALESCE(id::text = $1, false) IS NULL;
SELECT id FROM accounts WHERE (CASE WHEN id::text = $1 THEN token ELSE id END) IS NULL;
SELECT id FROM accounts WHERE (SELECT COALESCE(token, id) FROM accounts WHERE id::text = $1) IS NOT NULL;
SELECT id FROM accounts WHERE (CASE id::text WHEN $1 THEN 1 ELSE 0 END) IS NULL;
