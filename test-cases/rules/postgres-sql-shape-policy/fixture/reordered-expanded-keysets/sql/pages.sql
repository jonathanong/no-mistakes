-- The equality-prefix depth identifies each arm even when the OR arms are reversed.
SELECT * FROM accounts WHERE (a = $1 AND b > $2) OR a > $1 ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Unrelated predicates, mixed directions, and mismatched binds are not cursor expansions.
SELECT * FROM accounts WHERE (a = $1 AND b > $2) OR status = $3 ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE (a = $1 AND b > $2) OR a < $1 ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE (a = $1 AND b > $2) OR a > $9 ORDER BY a, b LIMIT $3;
