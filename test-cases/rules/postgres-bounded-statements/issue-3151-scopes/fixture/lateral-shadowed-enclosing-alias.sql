SELECT t.id, a.state FROM targets t
CROSS JOIN LATERAL (
  SELECT x.state FROM placements p
  JOIN LATERAL (
    SELECT recorded.state FROM activations recorded
    JOIN number_kinds t ON true
    WHERE recorded.placement_id = p.placement_id
      AND recorded.placement_revision = t.id
  ) x ON true
  WHERE p.placement_id = t.placement_id
) a WHERE t.id = $1;
