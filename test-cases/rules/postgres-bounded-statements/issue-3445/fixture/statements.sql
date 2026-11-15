-- Report: lower(mytype) is not the built-in text function.
SELECT id FROM custom WHERE lower(payload) = $1;
-- Report: lower of an integer is not the text built-in.
SELECT id FROM custom WHERE lower(n) = $1;
-- Pass: varchar still uses the built-in.
SELECT id FROM labels WHERE lower(name) = $1;
-- Pass: raw equality on an integer still implies NOT NULL.
SELECT id FROM counters WHERE n = $1;
