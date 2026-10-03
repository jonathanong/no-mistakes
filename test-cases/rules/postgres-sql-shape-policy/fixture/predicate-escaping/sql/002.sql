-- The first query in each pair matches the configured predicate; the second is a different value or expression.
SELECT * FROM accounts WHERE id > $1 AND status = E'a\' or x = \'b' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND (status = E'a' OR x = E'b') ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND status = U&'a\0027 or x = \0027b' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND (status = U&'a' OR x = U&'b') ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND score = 1E3 ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND score = 1E4 ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND status = $TAG$idle$TAG$ ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND status = $TAG$IDLE$TAG$ ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND flags = X'dead' ORDER BY id LIMIT $2;
SELECT * FROM accounts WHERE id > $1 AND flags = X'BEEF' ORDER BY id LIMIT $2;
