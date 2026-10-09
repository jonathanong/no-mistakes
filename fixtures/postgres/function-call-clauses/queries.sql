SELECT probe_select_list(), wrap(probe_select_arg()) FROM orders
WHERE wrap(probe_where()) AND EXISTS (SELECT probe_nested_select() FROM events WHERE probe_nested_where()) AND probe_restored_where()
GROUP BY probe_unscoped_group()
HAVING probe_having()
ORDER BY probe_order_by();

SELECT 1 FROM probe_from(probe_from_arg()) AS source
JOIN events ON probe_join_on();

VALUES (probe_values_left()) UNION ALL VALUES (probe_values_right());

SELECT array_agg(1 ORDER BY probe_argument_order()) FILTER (WHERE probe_filter_where());
SELECT count(1 WHERE probe_inline_where());
SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY probe_within_group_order());
SELECT sum(1) OVER (ORDER BY probe_window_order());
SELECT sum(1) OVER named FROM orders WINDOW named AS (ORDER BY probe_named_window_order());

-- A subquery SELECT list overrides its enclosing WHERE, then the outer context resumes.
SELECT 1 FROM orders WHERE (SELECT probe_nested_projection()) IS NOT NULL AND probe_after_subquery();
