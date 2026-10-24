SELECT 1;
-- The first opaque statement starts on line 3.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- The second opaque statement starts on line 6.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
