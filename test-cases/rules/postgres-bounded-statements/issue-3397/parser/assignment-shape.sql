SELECT (secs := $4);
SELECT callee((lower(x) := y));
SELECT make_interval((secs := $4));
SELECT make_interval(secs := other(a := 1));
UPDATE flows SET callback_error = NULL WHERE id = $1;
SELECT 1 FROM generate_series(start := $1, stop := 10) ids(id);
SELECT 1 FROM LATERAL generate_series(start := $1, stop := 10) AS ids(id);
