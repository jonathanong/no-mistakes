-- The grouping rewrite inserts punctuation before OFFSET.
SELECT id FROM items GROUP BY DISTINCT ROLLUP(id) OFFSET 2;
