-- Rows remain available, but omitted query context is never claimed complete.
INSERT INTO accounts WITH seed AS (SELECT 1) VALUES (1);
INSERT INTO accounts VALUES (1), (2) ORDER BY 1 LIMIT 1;
INSERT INTO accounts VALUES (1) OFFSET 1 FETCH FIRST 1 ROW ONLY;
INSERT INTO accounts VALUES (1) FOR UPDATE;
INSERT INTO accounts VALUES (1);
