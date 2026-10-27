-- A function null test needs its own predicate evidence; plain-column equality is insufficient.
SELECT 1 FROM partial_expr_nullguard e WHERE e.email = $1;
