SELECT LEFT(title, 80) FROM entries;

SELECT custom(title) FROM entries ORDER BY id LIMIT 26;

SELECT app.left(title, 80) FROM entries ORDER BY id LIMIT 26;

SELECT app.make_interval(secs => $1) FROM flows ORDER BY id LIMIT 1;

SELECT pg_catalog.left(title, 80) FROM entries ORDER BY id LIMIT 26;
