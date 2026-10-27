-- A wildcard cannot hide the explicit JSON expression next to it.
INSERT INTO orders (details, id)
SELECT jsonb_build_object('accountId', $1), source.* FROM (SELECT $2 AS id) source;
INSERT INTO orders (id, details)
SELECT source.*, jsonb_build_object('accountId', $1) FROM (SELECT $2 AS id) source;
-- A simple derived projection still writes this known JSON value.
INSERT INTO orders (details)
SELECT * FROM (SELECT jsonb_build_object('accountId', $1) AS details) source;
