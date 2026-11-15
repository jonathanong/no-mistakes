-- Pass: raw equality fixes lower(username) and implies username IS NOT NULL.
SELECT id FROM actors WHERE username = 'worker';
SELECT id FROM actors WHERE username = $1;
-- Pass: upper, one-argument btrim, and pg_catalog.lower.
SELECT label FROM labels WHERE label = $1;
SELECT title FROM titles WHERE title = $1;
SELECT handle FROM handles WHERE handle = $1;
-- Pass: both parts of UNIQUE (tenant_id, lower(email)) are fixed.
SELECT tenant_id FROM memberships WHERE tenant_id = $1 AND email = $2;
-- Pass: the cast target shares the column's equality family.
SELECT email FROM emails WHERE email = $1;
SELECT n FROM counters WHERE n = $1;
SELECT flag FROM flags WHERE flag = $1;
SELECT token FROM tokens WHERE token = $1;
-- Pass: a NOT NULL column makes null-safe equality fix lower(username).
SELECT username FROM required_names WHERE username IS NOT DISTINCT FROM $1;
-- Pass: a caller array is a finite pin of the same column.
SELECT label FROM labels WHERE label = ANY($1);
-- Pass: equality to a column of a bounded relation.
SELECT 1 FROM directories AS d JOIN normalized AS e ON e.email = d.email WHERE d.id = $1;
-- Pass: the same expression still matches when the column predicate is present.
SELECT id FROM actors WHERE lower(username) = $1 AND username IS NOT NULL;
-- Report: null-safe equality on a nullable column does not fix the expression.
SELECT id FROM actors WHERE username IS NOT DISTINCT FROM $1;
-- Report: function equality does not imply username IS NOT NULL.
SELECT id FROM actors WHERE lower(username) = $1;
-- Report: the reverse direction. lower(name) is not unique on raw name.
SELECT name FROM raw_names WHERE lower(name) = $1;
-- Report: length is not a recognized function.
SELECT name FROM sized WHERE name = $1;
-- Report: the partial predicate is not implied.
SELECT username FROM active_names WHERE username = $1;
-- Report: raw equality does not imply a predicate on the function.
SELECT username FROM strict_names WHERE username = $1;
-- Report: two-argument btrim is not a function of the column alone.
SELECT name FROM trimmed WHERE name = $1;
-- Report: a quoted name is a different function.
SELECT name FROM quoted_names WHERE name = $1;
-- Report: a function outside pg_catalog is not the built-in.
SELECT name FROM custom_names WHERE name = $1;
-- Report: char equality is not text equality.
SELECT code FROM codes WHERE code = $1;
-- Report: uuid cast to text is a different equality.
SELECT token FROM token_text WHERE token = $1;
-- Report: each composite part must be fixed.
SELECT tenant_id FROM memberships WHERE tenant_id = $1;
SELECT email FROM memberships WHERE email = $1;
-- Report: the argument is not a bare column.
SELECT name FROM wrapped WHERE name = $1;
-- Report: a positional alias list blocks the expression-key proof.
SELECT label FROM labels AS renamed(label) WHERE label = $1;
-- Report: the joined column comes from an unbounded relation.
SELECT 1 FROM directories AS d JOIN normalized AS e ON e.email = d.email;
-- Report: range lower can be null for a non-null empty range.
SELECT span FROM spans WHERE span = $1;
