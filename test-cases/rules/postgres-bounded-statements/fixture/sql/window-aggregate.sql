-- Window expressions can contain plain aggregates that preserve an implicit group.
SELECT row_number() OVER (ORDER BY count(*)) FROM orders WHERE false ORDER BY generate_series(1, 1000000);
-- Named window definitions need the same aggregate detection.
SELECT row_number() OVER w FROM orders WHERE false WINDOW w AS (ORDER BY count(*)) ORDER BY generate_series(1, 1000000);
-- Window references can inherit aggregate ordering from a named parent window.
SELECT row_number() OVER w2 FROM orders WHERE false WINDOW w AS (ORDER BY count(*)), w2 AS (w) ORDER BY generate_series(1, 1000000);
-- A named window without an aggregate must not preserve the implicit group.
SELECT row_number() OVER w FROM orders WHERE false WINDOW w AS (ORDER BY id) ORDER BY generate_series(1, 1000000);
