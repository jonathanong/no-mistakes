-- Positional aliases are syntax even when the source has no catalog-key identity.
SELECT * FROM (SELECT id, email FROM accounts) AS d(account_id, Email);
SELECT * FROM (SELECT id, email FROM accounts) AS "D"("Account.ID", "Email");
SELECT * FROM (SELECT id FROM accounts) AS d;
SELECT * FROM (SELECT id FROM accounts);
SELECT * FROM accounts a, LATERAL (SELECT a.id) AS d(account_id);
SELECT * FROM unnest(ARRAY[1, 2]) AS u(item);
SELECT * FROM accounts a(account_id);
-- Joined wrappers do not invent positional aliases for their child items.
SELECT 1 FROM (accounts JOIN orders ON accounts.id = orders.account_id) AS combined;
-- Unsupported SQL/JSON table shapes remain opaque without catalog-key alias credit.
SELECT * FROM JSON_TABLE($1::jsonb, '$[*]' COLUMNS (id integer PATH '$.id')) AS jt(item);
