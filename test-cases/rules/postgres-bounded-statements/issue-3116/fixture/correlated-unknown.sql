SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p CROSS JOIN unknown_rows u
  WHERE p.placement_id = target.placement_id AND active = $2
);

SELECT target.placement_id FROM image_placements target
WHERE target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p CROSS JOIN unknown_rows u
  WHERE p.placement_id = target.placement_id AND active = $2
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p CROSS JOIN unknown_rows u
  WHERE p.placement_id = p.placement_id AND active = $2
);

SELECT target.placement_id FROM image_placements target
WHERE target.placement_id = $1 AND target.created_by_id IN (
  SELECT p.created_by_id FROM image_placements p CROSS JOIN unknown_rows
  WHERE p.placement_id = target.placement_id AND active = $2
);
