-- An equality on archived_by_id reads only archived rows. It cannot satisfy
-- `archived_by_id IS NULL`, so the finding stays and explains the fix.
SELECT id FROM documents WHERE archived_by_id = $1 AND deleted_at IS NULL ORDER BY id;
