SELECT o.id, x.owner_id FROM owners o
CROSS JOIN LATERAL (
  SELECT (SELECT COALESCE(p.user_id, l.user_id)
          FROM placements p LEFT JOIN links l ON l.id = p.link_id
          WHERE p.placement_id = o.placement_id) AS owner_id
) x WHERE o.id = $1;
