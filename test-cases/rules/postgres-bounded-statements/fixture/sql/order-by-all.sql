-- DuckDB's ORDER BY ALL contains no expressions for cardinality analysis to inspect.
SELECT 1 FROM orders ORDER BY ALL;
