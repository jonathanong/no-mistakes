-- No mention of archived_by_id: the finding must not carry the equality hint.
SELECT id FROM documents WHERE owner_id = $1 AND deleted_at IS NULL;
