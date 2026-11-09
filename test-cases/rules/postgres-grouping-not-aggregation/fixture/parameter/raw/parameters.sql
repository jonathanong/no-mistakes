-- A real nullable unique column is still a valid DISTINCT key for COUNT.
SELECT COUNT(DISTINCT sql_placeholder_1) /* CASE: raw-real-nullable-unique */
FROM marker_inputs
GROUP BY sql_placeholder_1;

-- The parameter placeholder name must not alias the catalog column.
SELECT COUNT(DISTINCT $1) /* CASE: raw-bind-collision-negative */
FROM marker_inputs
GROUP BY sql_placeholder_1;
