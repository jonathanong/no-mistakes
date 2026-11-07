SELECT id FROM accounts WHERE id IS DISTINCT FROM COALESCE(token, id);
SELECT id FROM accounts WHERE id IS NOT DISTINCT FROM COALESCE(token, id);
SELECT a.id FROM accounts a WHERE COALESCE(a.token, a.id) IS DISTINCT FROM a.id;
SELECT a.id FROM accounts a WHERE COALESCE(a.token, a.id) IS NOT DISTINCT FROM a.id;
