SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.placement_id = target.placement_id
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.created_by_id = target.created_by_id
);

SELECT (SELECT p.created_by_id FROM image_placements p
        WHERE p.placement_id = target.placement_id)
FROM image_placements target WHERE target.placement_id = $1;

SELECT (SELECT p.created_by_id FROM image_placements p
        WHERE p.placement_id = target.placement_id)
FROM image_placements target;

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id IN (
  SELECT p.placement_id FROM image_placements p
  WHERE p.placement_id = target.placement_id
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.placement_id = missing.placement_id
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.placement_id = p.placement_id
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.external_ref = target.external_ref
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p
  WHERE p.placement_id = target.placement_id OR p.created_by_id = $2
);
