SELECT 1 WHERE id::text = $1;
SELECT 1 FROM accounts WHERE missing::text = $1;
SELECT 1 FROM (SELECT id FROM accounts) derived WHERE derived.id::text = $1;
SELECT 1 FROM accounts($1) a WHERE a.id::text = $2;
SELECT 1 FROM (accounts a JOIN orders o ON true) joined WHERE joined.id::text = $1;
SELECT 1 FROM accounts a JOIN accounts a ON true WHERE a.id::text = $1;
SELECT 1 FROM accounts a(alias_id, id) WHERE a.id::text = $1;
