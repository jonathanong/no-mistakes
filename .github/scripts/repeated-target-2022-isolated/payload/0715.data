WITH s AS (SELECT now()) SELECT count(*) FROM s;
WITH first AS (SELECT 'é' AS word), second AS (WITH nested AS (SELECT lower('é')) SELECT * FROM nested) SELECT count(*) FROM first, second;
SELECT now();
SELECT (SELECT now()) AS value;
(SELECT now()) ORDER BY 1;
