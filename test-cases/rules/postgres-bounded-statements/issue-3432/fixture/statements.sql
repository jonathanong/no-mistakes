-- Pass: strict lower(username) = lower($1) pins the expression index and implies username IS NOT NULL.
UPDATE tail_users
SET username = 'reclaimed-' || replace(id::text, '-', '')
WHERE lower(username) = lower($1) AND platform_account_kind IS NULL;
-- Pass: a bare parameter on the same strict call.
SELECT id FROM tail_users WHERE lower(username) = $1;
-- Pass: pg_catalog.lower matches an index written with the same function.
SELECT id FROM qualified_users WHERE pg_catalog.lower(username) = pg_catalog.lower($1);
-- Pass: upper and one-argument btrim are strict.
SELECT username FROM shaped_users WHERE upper(username) = $1;
SELECT username FROM trimmed_users WHERE btrim(username) = $1;
-- Report: coalesce is not strict and is not the indexed expression.
SELECT id FROM tail_users WHERE coalesce(username, '') = $1;
-- Report: the index predicate is not implied by the equality.
SELECT id FROM kind_users WHERE lower(username) = lower($1);
-- Report: a different function does not pin lower(username).
SELECT id FROM tail_users WHERE upper(username) = upper($1);
-- Report: null-safe equality can match a null username.
SELECT id FROM tail_users WHERE lower(username) IS NOT DISTINCT FROM lower($1);
-- Report: an extra argument is not one-argument btrim.
SELECT username FROM trimmed_users WHERE btrim(username, 'x') = $1;
-- Report: a quoted or other-schema function is not the built-in.
SELECT id FROM tail_users WHERE "lower"(username) = $1;
SELECT id FROM tail_users WHERE app.lower(username) = $1;
-- Report: a filter keeps the call from being the strict built-in.
SELECT id FROM tail_users WHERE lower(username) FILTER (WHERE true) = $1;
