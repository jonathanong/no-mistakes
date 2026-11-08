SELECT * FROM accounts WHERE id >= $1 AND id < $2 AND uuid_extract_timestamp(id) >= $3;
SELECT * FROM accounts WHERE uuid_extract_timestamp(id) >= $3;
SELECT * FROM accounts WHERE id >= $1 OR uuid_extract_timestamp(id) >= $3;
SELECT * FROM accounts a WHERE a.id = a.id AND fn(a.token) = $1;
SELECT * FROM accounts a WHERE a.email = $1 AND fn(a.token) = $2;
SELECT * FROM accounts a JOIN accounts b ON a.id = $1 WHERE fn(b.id) = $2;
SELECT * FROM accounts WHERE id = $1; SELECT * FROM accounts WHERE fn(id) = $2;
SELECT * FROM accounts a WHERE EXISTS (SELECT 1 FROM accounts b WHERE b.id = $1) AND fn(a.id) = $2;
SELECT * FROM accounts a WHERE NOT (a.id = $1) AND fn(a.id) = $2;
