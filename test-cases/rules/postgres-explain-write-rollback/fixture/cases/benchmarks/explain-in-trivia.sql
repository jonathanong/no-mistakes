/* outer /* inner */ EXPLAIN still in outer */
-- Explain
SELECT 'explain', E'it\'s EXPLAIN', $body$EXPLAIN ANALYZE$body$, "EXPLAIN",
       explain_count, caféexplain, field$explain;
SELECT (;
