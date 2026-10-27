-- Unparseable catalog expressions cannot establish a key proof.
SELECT 1 FROM broken_predicate b WHERE b.email = $1;
