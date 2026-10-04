-- A joined-table alias hides its children, so accounts.id below is an outer write reference.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM (accounts JOIN orders ON true) AS j LIMIT 1);
-- Without the joined-table alias, accounts.id resolves to the inner table.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM (accounts JOIN orders ON true) LIMIT 1);
-- A separate visible accounts source makes the qualifier local again.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM (accounts JOIN orders ON true) AS j, accounts LIMIT 1);
-- A hidden child's explicit alias also belongs to the outer target alias.
DELETE FROM accounts AS outer_a WHERE id IN (SELECT outer_a.id FROM (accounts AS outer_a JOIN orders ON true) AS j LIMIT 1);
-- Derived child aliases are hidden by the joined-table alias too.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM ((SELECT 7 AS id) accounts JOIN orders ON true) AS j LIMIT 1);
-- Nested joined-table aliases cannot resurrect their hidden child names.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM ((accounts JOIN orders ON true) AS j JOIN orders AS o ON true) AS k LIMIT 1);
-- The visible joined-table alias itself remains local.
DELETE FROM accounts WHERE id IN (SELECT j.account_id FROM (accounts JOIN orders ON true) AS j LIMIT 1);
-- A derived query establishes its own namespace below the outer joined-table alias.
DELETE FROM accounts WHERE id IN (SELECT j.account_id FROM ((SELECT accounts.id FROM accounts) a JOIN orders ON true) AS j LIMIT 1);
-- ON reads still resolve to children before the join alias hides them.
DELETE FROM accounts WHERE id IN (SELECT j.account_id FROM (accounts JOIN orders ON accounts.id = orders.id) AS j LIMIT 1);
-- LATERAL inside the joined group sees preceding children, even after they disappear outward.
DELETE FROM accounts WHERE id IN (SELECT j.child_id FROM (accounts JOIN LATERAL (SELECT accounts.id AS child_id) AS a ON true) AS j LIMIT 1);
-- A later, differently named source must not resurrect the hidden child's namespace.
DELETE FROM accounts WHERE id IN (SELECT accounts.id FROM (accounts JOIN orders ON true) AS j, orders AS later LIMIT 1);
-- Whole-row child references are hidden along with column qualifiers.
DELETE FROM accounts WHERE id IN (SELECT (accounts).id FROM (accounts JOIN orders ON true) AS j LIMIT 1);
-- Quoted relation components retain the same hidden namespace behavior.
DELETE FROM "accounts" WHERE id IN (SELECT "accounts".id FROM ("accounts" JOIN orders ON true) AS "j" LIMIT 1);
-- Whole-row names inside ON remain child-local too.
DELETE FROM accounts WHERE id IN (SELECT j.account_id FROM (accounts JOIN orders ON (accounts).id = orders.account_id) AS j LIMIT 1);
-- Catalog-qualified child references also remain local inside ON and LATERAL.
DELETE FROM public.accounts WHERE id IN (SELECT j.account_id FROM (public.accounts JOIN orders ON public.accounts.id = orders.account_id) AS j LIMIT 1);
DELETE FROM public.accounts WHERE id IN (SELECT j.child_id FROM (public.accounts JOIN LATERAL (SELECT public.accounts.id AS child_id) AS a ON true) AS j LIMIT 1);
