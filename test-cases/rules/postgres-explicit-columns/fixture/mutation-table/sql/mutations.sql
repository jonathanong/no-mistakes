-- TABLE right arms retain physical relation identity inside mutation expressions.
UPDATE orders SET status = 'x' WHERE EXISTS (SELECT id FROM safe UNION ALL TABLE public.topics);
DELETE FROM orders WHERE EXISTS (SELECT id FROM safe UNION ALL TABLE public."Topics");
UPDATE orders SET id = (SELECT id FROM safe UNION ALL TABLE public.topics);
DELETE FROM orders RETURNING (SELECT id FROM safe UNION ALL TABLE public.topics);
UPDATE orders SET status = 'x' FROM (SELECT id FROM safe UNION ALL TABLE public.topics) AS source;
DELETE FROM orders USING (SELECT id FROM safe UNION ALL TABLE public.topics) AS source;
-- Visible CTE names stay local even inside an executed mutation query body.
WITH topics AS (SELECT id FROM safe) DELETE FROM orders WHERE EXISTS (SELECT id FROM safe UNION ALL TABLE topics);
-- Schema qualification still makes the TABLE arm physical beside a same-named CTE.
WITH topics AS (SELECT id FROM safe) UPDATE orders SET status = 'x' WHERE EXISTS (SELECT id FROM safe UNION ALL TABLE public.topics);
