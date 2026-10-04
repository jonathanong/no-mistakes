-- int4 aliases on a bind preserve its identity in the expanded prefix chain.
SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Different binds, a narrowing cast, and an unknown custom cast remain conservative.
SELECT * FROM accounts WHERE a > $1::int OR (a = $9 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::smallint OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::cursor_id OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Opaque casts still match when every occurrence uses the identical expression.
SELECT * FROM accounts WHERE a > $1::cursor_id OR (a = $1::cursor_id AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::smallint OR (a = $1::smallint AND b > $2) ORDER BY a, b LIMIT $3;
-- In standalone SQL, marker-like names are ordinary columns, not cursor binds.
SELECT * FROM accounts WHERE a > sql_placeholder_1::int OR (a = sql_placeholder_1 AND b > sql_placeholder_2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::int OR (a = 1 AND b > $2) ORDER BY a, b LIMIT $3;
-- An outer int cast cannot erase a value-changing inner cast; identical chains remain comparable.
SELECT * FROM accounts WHERE a > ($1::numeric)::int OR (a = $1::numeric AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > ($1::numeric)::int OR (a = ($1::numeric)::int AND b > $2) ORDER BY a, b LIMIT $3;
-- A declared numeric bind can change value under int4 conversion, even without a nested cast.
PREPARE numeric_cursor(numeric, numeric, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- A composite key can use an identical tuple of bind values as its cursor threshold.
SELECT * FROM accounts WHERE a > ($1, $2) OR (a = ($1, $2) AND b > $3) ORDER BY a, b LIMIT $4;
-- A declared int4 parameter keeps the transparent cast safe.
PREPARE integer_cursor(integer, integer, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Even a declared numeric bind can use an identical cast on both occurrences.
PREPARE numeric_cast_cursor(numeric, numeric, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1::int AND b > $2) ORDER BY a, b LIMIT $3;
