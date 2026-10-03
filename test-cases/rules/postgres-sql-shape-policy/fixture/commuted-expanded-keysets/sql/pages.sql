-- Commuted scalar comparison operands are equivalent to the tuple cursor.
SELECT * FROM accounts WHERE $1 < a OR ($1 = a AND $2 < b) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE ($1, $2) < (a, b) ORDER BY a, b LIMIT $3;
-- Mixed directions, mismatched equality binds, and unrelated columns stay conservative.
SELECT * FROM accounts WHERE $1 < a OR ($1 = a AND $2 > b) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE $1 < a OR ($9 = a AND $2 < b) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE $1 < a OR ($1 = a AND status < $2) ORDER BY a, b LIMIT $3;
