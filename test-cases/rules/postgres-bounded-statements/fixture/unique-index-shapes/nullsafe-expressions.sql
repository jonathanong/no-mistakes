-- Null-safe expression equality needs the index's NULLS NOT DISTINCT contract.
SELECT 1 FROM nd_emails e WHERE lower(e.email) IS NOT DISTINCT FROM $1;
SELECT 1 FROM normalized_emails e WHERE lower(e.email) IS NOT DISTINCT FROM $1;
