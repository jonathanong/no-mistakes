-- This saved DuckDB syntax exercises keyword ALL in the shared query AST.
-- It must not be treated as a bare outer-column read; PostgreSQL support is unchanged.
SELECT 1 GROUP BY ALL ORDER BY ALL;
