SELECT 1 FROM accounts a JOIN other o ON true WHERE id::text = $1;
SELECT 1 FROM public.accounts a WHERE a.id::text = $1;
SELECT 1 FROM audit.accounts a WHERE a.id::text = $1;
SELECT 1 FROM accounts a JOIN orders o ON true WHERE id::text = $1;
SELECT 1 FROM accounts a JOIN missing o ON true WHERE id::text = $1;
SELECT 1 FROM accounts a WHERE accounts.id::text = $1;
-- A column alias list changes the catalog correspondence: a.id is the text email column.
SELECT 1 FROM accounts a(alias_id, id) WHERE a.id::text = $1;
WITH accounts AS (SELECT id FROM text_ids) SELECT 1 FROM accounts WHERE id::text = $1;
SELECT 1 FROM dupes WHERE id::text = $1;
SELECT 1 FROM public.dupes WHERE id::text = $1;
SELECT 1 FROM accounts a JOIN orders o ON later.id::text = $1 JOIN accounts later ON true;
SELECT 1 FROM accounts a JOIN accounts b ON true WHERE id::text = $1;
