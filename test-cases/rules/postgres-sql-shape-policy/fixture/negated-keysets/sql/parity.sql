-- Even NOT parity retains NULL-switchable cursors; odd parity does not.
SELECT * FROM accounts WHERE NOT NOT ($1 IS NULL OR id > $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT (NOT (NOT (NOT ($1 IS NULL OR id > $1)))) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT NOT NOT ($1 IS NULL OR id > $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT NOT (id > $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT NOT NOT (id > $1) ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE NOT NOT (id = $1) ORDER BY id LIMIT $2;
