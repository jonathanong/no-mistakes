-- Recovered interpolations have no declared positional parameter index.
SELECT * FROM accounts WHERE a > sql_placeholder_1::int OR (a = sql_placeholder_1 AND b > sql_placeholder_2) ORDER BY a, b LIMIT 10;
PREPARE unknown_interpolation(integer) AS SELECT * FROM accounts WHERE a > sql_placeholder_1::int OR (a = sql_placeholder_1 AND b > sql_placeholder_2) ORDER BY a, b LIMIT 10;
SELECT * FROM accounts WHERE a > (sql_placeholder_1::int, sql_placeholder_2) OR (a = (sql_placeholder_1, sql_placeholder_2) AND b > sql_placeholder_3) ORDER BY a, b LIMIT 10;
