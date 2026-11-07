SELECT t.id FROM accounts t JOIN unnest($1::uuid[]) AS req(id) ON t.id = req.id;
WITH picked AS (SELECT id FROM accounts WHERE id = $1) SELECT t.id FROM accounts t JOIN picked ON t.id = picked.id;
SELECT t.id FROM accounts t JOIN unnest($1::text[]) AS req(name) ON t.name = req.name;
