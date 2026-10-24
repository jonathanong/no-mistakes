-- An unpinned source joined through a non-unique target_id does not bound writes.
UPDATE joined_targets AS t SET payload = $1 FROM joined_sources AS s WHERE t.id = s.target_id AND s.tenant_id = $2;
DELETE FROM joined_targets AS t USING joined_sources AS s WHERE t.id = s.target_id AND s.tenant_id = $1;
-- A target key pinned directly remains bounded even with an unbounded source.
UPDATE joined_targets AS t SET payload = $1 FROM joined_sources AS s WHERE t.id = $2 AND t.id = s.target_id AND s.tenant_id = $3;
DELETE FROM joined_targets AS t USING joined_sources AS s WHERE t.id = $1 AND t.id = s.target_id AND s.tenant_id = $2;
-- A bounded joined source can pin the target's unique key.
UPDATE joined_targets AS t SET payload = $1 FROM joined_sources AS s WHERE t.id = s.id AND s.id = $2;
DELETE FROM joined_targets AS t USING (SELECT id FROM joined_sources ORDER BY id LIMIT 1) AS s WHERE t.id = s.id;
-- The unique root_id hop and target primary key carry one root to at most one target.
UPDATE joined_targets AS t SET payload = $1 FROM joined_roots AS r JOIN joined_links AS l ON l.root_id = r.id WHERE t.id = l.target_id AND r.id = $2;
-- The root's bound cannot be lent to this unbounded sibling that selects targets.
DELETE FROM joined_targets AS t USING joined_roots AS r JOIN joined_links AS l ON l.root_id = r.id CROSS JOIN joined_sources AS s WHERE r.id = $1 AND t.id = s.target_id;
