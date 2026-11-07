SELECT o.id FROM orders o JOIN account_view v ON COALESCE(o.account_id, o.id) = v.token::uuid;
SELECT o.id FROM orders o JOIN computed_account_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN unknown_account_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN recursive_account_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o RIGHT JOIN account_view v ON v.id = COALESCE(o.account_id, o.id);
SELECT o.id FROM orders o JOIN account_view v ON COALESCE(v.id, o.id) = o.account_id;
