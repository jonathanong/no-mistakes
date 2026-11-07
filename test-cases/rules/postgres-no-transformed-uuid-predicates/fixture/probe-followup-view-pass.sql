SELECT o.id FROM orders o JOIN account_view v ON v.id = COALESCE(o.account_id, o.id) WHERE o.id = $1;
SELECT o.id FROM orders o LEFT JOIN account_view v ON COALESCE(o.account_id, o.id) = v.id WHERE o.id = $1;
SELECT o.id FROM orders o JOIN renamed_account_view v ON v.account_key = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN quoted_account_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN quoted_alias_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN public.account_view v ON v.id = COALESCE(o.account_id, o.id);
