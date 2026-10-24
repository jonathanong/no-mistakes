-- Alias lists must not reinterpret the physical key or an expression target.
SELECT 1 FROM nd_contacts n(label, email) WHERE n.email IS NOT DISTINCT FROM $1;
SELECT 1 FROM nd_contacts n(email, label) WHERE n.email IS NOT DISTINCT FROM $1;
SELECT 1 FROM normalized_emails e(email) WHERE lower(e.email) = $1;
-- A function null test needs its own predicate evidence; plain-column equality is insufficient.
SELECT 1 FROM partial_expr_nullguard e WHERE e.email = $1;
SELECT 1 FROM partial_expr_nullguard e WHERE e.email = $1 AND lower(e.email) IS NOT NULL;
