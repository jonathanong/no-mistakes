-- Unparseable catalog expressions cannot establish a key proof.
SELECT 1 FROM broken_predicate b WHERE b.email = $1;
SELECT 1 FROM broken_expression b WHERE lower(b.email) = $1;
