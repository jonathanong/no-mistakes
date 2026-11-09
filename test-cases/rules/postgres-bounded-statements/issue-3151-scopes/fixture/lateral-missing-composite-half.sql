SELECT t.id, a.state FROM targets t
CROSS JOIN LATERAL (
  SELECT x.state FROM placements p
  JOIN LATERAL (
    SELECT recorded.state FROM activations recorded
    WHERE recorded.placement_id = p.placement_id
  ) x ON true
  WHERE p.placement_id = t.placement_id
) a WHERE t.id = $1;
