-- Group-only names absent from an arm's projection can read the outer row.
DELETE FROM accounts a WHERE a.id IN (SELECT 1 GROUP BY id UNION ALL SELECT 1 LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT 1 UNION ALL SELECT 1 GROUP BY id LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT 1 GROUP BY (id + 1) UNION ALL SELECT 1 LIMIT 1);
-- Actual explicit output labels remain local, including quoted labels.
DELETE FROM accounts a WHERE a.id IN (SELECT $1 AS id GROUP BY id UNION ALL SELECT 1 LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT $1 AS "id" GROUP BY "id" UNION ALL SELECT 1 LIMIT 1);
-- A projected outer identifier still reads the outer row despite sharing its grouping label.
DELETE FROM accounts a WHERE a.id IN (SELECT id GROUP BY id UNION ALL SELECT 1 LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT a.id GROUP BY id UNION ALL SELECT 1 LIMIT 1);
-- A grouping column owned by a physical or derived source remains independent.
DELETE FROM accounts a WHERE a.id IN (SELECT 1 FROM orders GROUP BY id UNION ALL SELECT 1 LIMIT 1);
DELETE FROM accounts a WHERE a.id IN (SELECT 1 FROM (SELECT $1 AS id) d GROUP BY id UNION ALL SELECT 1 LIMIT 1);
