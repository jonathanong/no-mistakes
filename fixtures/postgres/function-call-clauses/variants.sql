SELECT probe_projection() AS original;
SELECT sum(1) OVER named FROM orders WINDOW named AS (ORDER BY probe_named_order());
SELECT array_agg(1);
