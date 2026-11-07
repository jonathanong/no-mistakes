SELECT id FROM accounts WHERE COALESCE(token, id) IS DISTINCT FROM $1;
SELECT id FROM accounts WHERE COALESCE(token, id) IS NOT DISTINCT FROM $1;
SELECT id FROM accounts WHERE $1 IS DISTINCT FROM COALESCE(token, id);
SELECT id FROM accounts WHERE $1 IS NOT DISTINCT FROM COALESCE(token, id);
SELECT id FROM accounts WHERE COALESCE(token, id) IS DISTINCT FROM id::uuid;
SELECT id FROM accounts WHERE email IS DISTINCT FROM COALESCE(token, id)::text;
SELECT a.id FROM accounts a CROSS JOIN computed_account_view v WHERE id IS DISTINCT FROM COALESCE(a.token, a.id);
SELECT a.id FROM accounts a CROSS JOIN partial_account_view v WHERE id IS DISTINCT FROM COALESCE(a.token, a.id);
