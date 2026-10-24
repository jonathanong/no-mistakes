-- The unknown COPY option makes only the header unanalyzable; rows remain data.
COPY accounts (id, note) FROM STDIN WITH (frobulous true);
1	SELECT 'unterminated;
\.
FROBULATE accounts;
