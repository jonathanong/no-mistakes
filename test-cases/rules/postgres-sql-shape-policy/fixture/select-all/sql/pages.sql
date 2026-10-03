-- ALL preserves duplicate rows and must remain equivalent to an ordinary SELECT.
SELECT ALL account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2;
SELECT account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2;
SELECT DISTINCT account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2;
SELECT DISTINCT ON (account_id) account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2;
SELECT ALL account_id FROM orders ORDER BY account_id LIMIT $1;
SELECT ALL account_id AS id FROM orders ORDER BY id LIMIT $1;
