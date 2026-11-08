SELECT p.placement_id FROM image_placements target
CROSS JOIN LATERAL (SELECT placement_id) child
JOIN image_placements p ON p.placement_id = child.placement_id;

SELECT p.placement_id FROM image_placements target
CROSS JOIN LATERAL (SELECT target.placement_id AS placement_id) child
JOIN image_placements p ON p.placement_id = child.placement_id
WHERE target.placement_id = $1;
