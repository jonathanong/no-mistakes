VALUES (1);
(SELECT 1 FROM accounts WHERE id::text = $1) UNION ALL (SELECT 1 FROM text_ids WHERE id::text = $1);
