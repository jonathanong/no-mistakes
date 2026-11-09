SELECT DISTINCT id /* CASE: grouping-negative-repeated-pinned */ FROM posts WHERE id=$1::uuid GROUP BY GROUPING SETS((id),(id));

SELECT id /* CASE: grouping-negative-rollup-removal-pinned */ FROM posts WHERE id=$1::uuid GROUP BY ROLLUP(id);

SELECT id /* CASE: grouping-negative-cube-removal-pinned */ FROM posts WHERE id=$1::uuid GROUP BY CUBE(id);

SELECT id /* CASE: grouping-negative-repeated-removal-pinned */ FROM posts WHERE id=$1::uuid GROUP BY GROUPING SETS((id),(id));

SELECT DISTINCT slug /* CASE: grouping-negative-nullable-rollup */ FROM nullable_equal_slugs GROUP BY ROLLUP(slug);

SELECT DISTINCT slug /* CASE: grouping-negative-nullable-cube */ FROM nullable_equal_slugs GROUP BY CUBE(slug);

SELECT DISTINCT slug /* CASE: grouping-singleton-nullable */ FROM nullable_equal_slugs GROUP BY GROUPING SETS((slug));

SELECT DISTINCT slug /* CASE: grouping-rollup-filter-nonnull */ FROM nullable_equal_slugs WHERE slug IS NOT NULL GROUP BY ROLLUP(slug);

SELECT DISTINCT slug /* CASE: grouping-cube-strict-equality */ FROM nullable_equal_slugs WHERE slug=$1::text GROUP BY CUBE(slug);

SELECT slug /* CASE: grouping-singleton-removal */ FROM nullable_equal_slugs WHERE slug=$1::text GROUP BY GROUPING SETS((slug));
