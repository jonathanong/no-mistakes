SELECT 1;
-- Two unsupported statements must retain separate starts after a valid sibling.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- A second failure must not collapse into the first finding.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
