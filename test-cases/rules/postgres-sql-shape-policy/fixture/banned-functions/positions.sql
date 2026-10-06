SELECT pg_sleep(1);
SELECT pg_catalog.pg_sleep_for('1 second');
WITH timer AS (SELECT pg_sleep_until(now())) SELECT * FROM timer;
SELECT (SELECT pg_sleep(1));
SELECT CASE WHEN true THEN pg_sleep(1) ELSE NULL END;
SELECT 1 WHERE pg_sleep(1) IS NULL;
SELECT coalesce(pg_sleep(1), pg_sleep(2));
SELECT sum(1) OVER (ORDER BY pg_sleep(1));
SELECT * FROM pg_sleep(1);
SELECT * FROM LATERAL pg_catalog.pg_sleep(1);
UPDATE items SET value = pg_sleep(1) WHERE id = 1;
INSERT INTO items(value) VALUES(pg_sleep(1)) RETURNING pg_sleep(2);
SELECT pg_sleep AS pg_sleep_until FROM items AS pg_sleep_for;
SELECT 'pg_sleep(1)' AS text;
-- pg_sleep(1) is only text here.
