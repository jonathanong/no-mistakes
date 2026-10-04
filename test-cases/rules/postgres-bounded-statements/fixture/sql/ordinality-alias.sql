-- One alias always names an ordinary output; ordinality is appended afterward, even for unknown-width functions.
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM app.ids() WITH ORDINALITY AS f(value) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT f.ordinality FROM app.ids() WITH ORDINALITY AS f(value) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT "ordinality" FROM app.ids() WITH ORDINALITY AS f("value") LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM app.ids() WITH ORDINALITY AS f(value));
-- Two aliases could rename the ordinality column of a one-column function; do not invent its output width.
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM app.ids() WITH ORDINALITY AS f(value, position) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM app.ids() WITH ORDINALITY AS f LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM app.ids() AS f(value) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT id FROM app.ids() WITH ORDINALITY AS f(value) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT ordinality FROM pg_catalog.generate_series(1, 10) WITH ORDINALITY AS f(value) LIMIT 1);
DELETE FROM accounts WHERE id IN (SELECT position FROM app.ids() WITH ORDINALITY AS f(value, position) LIMIT 1);
