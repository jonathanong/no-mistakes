-- The expression equality alone does not prove function strictness to the analyzer.
SELECT 1 FROM partial_emails AS e WHERE lower(e.email) = $1;
