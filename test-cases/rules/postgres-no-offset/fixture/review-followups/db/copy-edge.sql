-- Header nesting, quotes, and comments must not confuse COPY data ownership.
SELECT 'STDIN', E'escaped\'quote;', "quoted""identifier";
/* outer /* nested ; */ comment */
COPY (SELECT id FROM stdin OFFSET 1) TO STDOUT;
SELECT $tag$STDIN; opaque$tag$, $1, name$identifier;
COPY t (id) FROM 'STDIN';
COPY t (id) FROM STDIN;
quotes ' and ; and 雪 are data
\.
SELECT (SELECT 1 OFFSET 0) OFFSET 2;
