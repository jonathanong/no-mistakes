SELECT 1 FROM orders d JOIN audit.accounts p ON p.id = d.id::text;
SELECT 1 FROM orders d JOIN accounts p ON p.id = COALESCE(d.account_id, d.id);
SELECT 1 FROM orders d JOIN public.accounts p ON p.id = COALESCE(d.account_id, d.id);
SELECT 1 FROM orders d JOIN public.dupes p ON p.id = COALESCE(d.account_id, d.id);
