-- The complete equality-prefix chain is a cursor; mixed directions and binds are not.
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a < $1 OR (a = $1 AND b < $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $9 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR b > $2 ORDER BY a, b LIMIT $3;
-- Both queries below are selective because the expanded keys do not follow ORDER BY.
SELECT * FROM accounts WHERE b > $1 OR (b = $1 AND a > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b > $2) ORDER BY a ASC, b DESC LIMIT $3;
SELECT * FROM accounts WHERE a < $1 OR (a = $1 AND b < $2) ORDER BY a DESC, b DESC LIMIT $3;
