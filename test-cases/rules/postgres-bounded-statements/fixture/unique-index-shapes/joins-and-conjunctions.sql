-- Partial predicates must constrain the same scan, including through bounded joins.
UPDATE community_members m SET removed_at = now() WHERE m.community_id = $1 AND m.user_id = $2 AND m.removed_at IS NULL;
UPDATE community_members m SET removed_at = now() WHERE m.community_id = $1 AND m.user_id = $2;
SELECT 1 FROM community_members m JOIN directories d ON m.community_id = d.id AND m.user_id = $2 AND m.removed_at IS NULL WHERE d.id = $1;
SELECT 1 FROM community_members m JOIN directories d ON m.community_id = d.id AND m.user_id = $2 WHERE d.id = $1;
-- ON cannot prove the partial predicate for the preserved side of a LEFT JOIN.
SELECT 1 FROM community_members m LEFT JOIN directories d ON d.id = $3 AND m.removed_at IS NULL WHERE m.community_id = $1 AND m.user_id = $2;
SELECT 1 FROM directories d LEFT JOIN community_members m ON m.community_id = d.id AND m.user_id = $2 AND m.removed_at IS NULL WHERE d.id = $1;
SELECT 1 FROM partial_multi m WHERE m.slug = $1 AND m.archived = false AND m.active = true;
SELECT 1 FROM partial_multi m WHERE m.slug = $1 AND m.active = true;
SELECT 1 FROM partial_multi m WHERE m.slug = $1 AND m.active = true AND (m.archived = false OR m.slug = $2);
-- Positional aliases swap active and archived; the visible predicate is not the physical predicate.
SELECT 1 FROM partial_docs pd(id, slug, archived, active) WHERE pd.slug = $1 AND pd.active = true;
