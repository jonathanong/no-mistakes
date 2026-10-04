-- Safe casts use the referenced bind declaration, including recursively nested tuples.
PREPARE mixed_cursor(integer, text, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
PREPARE reordered_cursor(text, integer, integer) AS SELECT * FROM accounts WHERE a > $2::int OR (a = $2 AND b > $1) ORDER BY a, b LIMIT $3;
PREPARE qualified_cursor(pg_catalog.int4, text, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::pg_catalog.int4 OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::"pg_catalog"."int4" OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > ($1::int, $2) OR (a = ($1, $2) AND b > $3) ORDER BY a, b LIMIT $4;
PREPARE tuple_cursor(integer, text, integer, integer) AS SELECT * FROM accounts WHERE a > ($1::int, $2) OR (a = ($1, $2) AND b > $3) ORDER BY a, b LIMIT $4;
PREPARE nested_tuple_cursor(integer, integer, text, integer, integer) AS SELECT * FROM accounts WHERE a > ($1, ($2::int, $3)) OR (a = ($1, ($2, $3)) AND b > $4) ORDER BY a, b LIMIT $5;
-- Value-changing, missing-type, lookalike, and mismatched bind identities remain opaque.
PREPARE numeric_cursor(numeric, text, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
PREPARE text_cursor(text, integer, integer) AS SELECT * FROM accounts WHERE a > $1::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
PREPARE missing_type_cursor(integer, text) AS SELECT * FROM accounts WHERE a > $4::int OR (a = $4 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::app.int4 OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::pg_catalog."INT4" OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > $1::"PG_CATALOG".int4 OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
PREPARE numeric_tuple_cursor(numeric, text, integer, integer) AS SELECT * FROM accounts WHERE a > ($1::int, $2) OR (a = ($1, $2) AND b > $3) ORDER BY a, b LIMIT $4;
SELECT * FROM accounts WHERE a > ($1::numeric)::int OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
SELECT * FROM accounts WHERE a > ($1::int, $2) OR (a = ($1, $3) AND b > $4) ORDER BY a, b LIMIT $5;
SELECT * FROM accounts WHERE a > sql_placeholder_1::int OR (a = sql_placeholder_1 AND b > $2) ORDER BY a, b LIMIT $3;
-- Parser-accepted non-scalar casts and type modifiers never supply transparent bind proof.
SELECT * FROM accounts WHERE a > ($1, $2)::int OR (a = ($1, $2) AND b > $3) ORDER BY a, b LIMIT $4;
SELECT * FROM accounts WHERE a > $1::pg_catalog.int4(2) OR (a = $1 AND b > $2) ORDER BY a, b LIMIT $3;
PREPARE zero_index_cursor(integer, integer, integer) AS SELECT * FROM accounts WHERE a > $0::int OR (a = $0 AND b > $2) ORDER BY a, b LIMIT $3;
