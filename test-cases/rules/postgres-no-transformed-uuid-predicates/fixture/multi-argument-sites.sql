SELECT 1 FROM accounts a JOIN orders o ON true WHERE fn_current_screening(a.id, o.id) = $1;
SELECT 1 FROM accounts a JOIN orders o ON true WHERE COALESCE(a.id, a.token, o.id, o.account_id) = $1;
SELECT 1 FROM accounts a JOIN accounts b ON true JOIN orders o ON true JOIN orders p ON true WHERE num_nonnulls(COALESCE(a.id, a.token, b.id, b.token), COALESCE(o.id, o.account_id, p.id, p.account_id)) = 1;
SELECT 1 FROM accounts WHERE fn(id, token) = $1 AND fn(id, token) = $2;
SELECT 1 FROM accounts WHERE fn(
  id,
  token
) = $1;
SELECT 1 FROM accounts WHERE fn(id, id, email) = $1;
SELECT 1 FROM accounts WHERE id::text = $1 AND token::text = $2;
SELECT 1 FROM accounts WHERE fn(id) = $1 AND fn(id) = $2;
SELECT 1 FROM accounts WHERE fn(
  -- no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates -- This argument intentionally transforms the UUID.
  id,
  token
) = $1;
SELECT 1 FROM accounts WHERE COALESCE(id, token) IS NULL;
SELECT 1 FROM accounts WHERE COALESCE(id = $1, token = $2, false);
SELECT 1 FROM accounts WHERE COALESCE(fn(id, token) = $1, false);
SELECT 1 FROM accounts WHERE fn(id = $1, token = $2) = true;
SELECT 1 FROM accounts WHERE CAST(CASE WHEN id = $1 AND token = $2 THEN email ELSE email END AS text) = $3;
