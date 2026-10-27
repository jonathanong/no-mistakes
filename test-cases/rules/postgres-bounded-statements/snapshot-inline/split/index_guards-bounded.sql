SELECT 1 FROM nd_contacts n(email, label) WHERE n.email IS NOT DISTINCT FROM $1;
SELECT 1 FROM partial_expr_nullguard e WHERE e.email = $1 AND lower(e.email) IS NOT NULL;
