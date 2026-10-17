-- Source locks have no typed projection, even in a nested source query.
WITH changed AS (INSERT INTO dst SELECT id FROM src FOR UPDATE RETURNING id)
SELECT id FROM changed;
WITH changed AS (INSERT INTO dst SELECT id FROM (SELECT id FROM src FOR SHARE) nested RETURNING id)
SELECT id FROM changed;
-- Preserve the established ordinary SELECT contract outside INSERT sources.
SELECT id FROM src FOR UPDATE;
WITH changed AS (INSERT INTO dst SELECT id FROM src RETURNING id)
SELECT id FROM changed FOR UPDATE;
