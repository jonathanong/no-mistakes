WITH RECURSIVE r AS (SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT a.id FROM accounts a JOIN r ON a.id = r.id) SELECT 1 FROM r;
