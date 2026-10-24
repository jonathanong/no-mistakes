COPY accounts (id, note) FROM STDIN;
\.
COPY accounts (id, note) FROM STDIN;
1	SELECT 'unterminated;
2	"quoted; SELECT 1;
3	/* unterminated ;
\.
FROBULATE accounts;
