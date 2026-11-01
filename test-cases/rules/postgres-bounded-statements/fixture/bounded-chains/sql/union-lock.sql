SELECT p.id, b.owner_id, b.asset_id
FROM (
  SELECT placement_id, owner_id, asset_id FROM owner_placements
  UNION ALL
  SELECT placement_id, NULL::uuid, asset_id FROM surface_placements
) b
JOIN placements p ON p.id = b.placement_id
WHERE b.owner_id = ANY($1::uuid[]) OR b.asset_id = ANY($2::uuid[]) OR p.id = ANY($3::uuid[])
ORDER BY p.id FOR UPDATE OF p
