-- The expression equality alone does not prove function strictness to the analyzer.
SELECT 1 FROM partial_emails AS e WHERE lower(e.email) = $1;
SELECT 1 FROM partial_emails AS e WHERE e.email IS NOT NULL AND lower(e.email) = $1;
SELECT 1 FROM partial_emails AS e WHERE lower(e.email) IS NOT DISTINCT FROM $1;
SELECT 1 FROM partial_emails AS e WHERE e.email IS NULL AND lower(e.email) IS NOT DISTINCT FROM NULL;
SELECT 1 FROM partial_emails AS e WHERE e.email IS NOT NULL AND lower(e.email || '') = $1;
