-- Wrapped self-owned arrays retain dependencies without finite-key credit.
DELETE FROM accounts WHERE id = ANY(account_ids::uuid[]);
DELETE FROM accounts WHERE id = ANY(coalesce(account_ids, $1));
DELETE FROM accounts a WHERE a.id = ANY(coalesce(a.account_ids::uuid[], $1));
DELETE FROM accounts WHERE id = ANY($1::uuid[]);
DELETE FROM accounts WHERE id = ANY(coalesce($1, $2));
DELETE FROM accounts WHERE id = ANY(missing.account_ids::uuid[]);
DELETE FROM accounts WHERE id = account_ids;
DELETE FROM accounts WHERE accounts.id = ANY(accounts.account_ids::uuid[]);
