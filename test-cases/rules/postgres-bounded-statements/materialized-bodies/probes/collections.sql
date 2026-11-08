SELECT k.id FROM lookup_rows k WHERE k.id = ANY(ARRAY(SELECT e.user_id FROM edge_rows e WHERE e.group_id = k.group_id)) LIMIT 1;
SELECT k.id FROM lookup_rows k WHERE k.id IN (SELECT e.user_id FROM edge_rows e WHERE e.group_id = k.group_id) LIMIT 1;
SELECT (SELECT e.id FROM edge_rows e WHERE e.user_id = k.user_id) = ANY(SELECT user_id FROM edge_rows) FROM lookup_rows k LIMIT 1;
SELECT k.id FROM lookup_rows k WHERE k.id = ALL(SELECT user_id FROM edge_rows) LIMIT 1;
