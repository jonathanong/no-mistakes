-- int4 aliases on a bind preserve its identity in the expanded prefix chain.
SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Different binds, a narrowing cast, and an unknown custom cast remain conservative.
SELECT * FROM accounts WHERE a > $1::int OR (a = $9 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::smallint OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::cursor_id OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Opaque casts still match when every occurrence uses the identical expression.
SELECT * FROM accounts WHERE a > $1::cursor_id OR (a = $1::cursor_id AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::smallint OR (a = $1::smallint AND b > $2) ORDER BY a, b LIMIT $3;
-- Recovered SQL placeholders are identifiers; a literal prefix cannot copy a bind.
SELECT * FROM accounts WHERE a > sql_placeholder_1::int OR (a = sql_placeholder_1 AND b > sql_placeholder_2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::int OR (a = 1 AND b > $2) ORDER BY a, b LIMIT $3;
