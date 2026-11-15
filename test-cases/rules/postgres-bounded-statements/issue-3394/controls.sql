SELECT id FROM bindings WHERE id = $1;
SELECT id FROM bindings WHERE lineage_id = $1 LIMIT 1;
