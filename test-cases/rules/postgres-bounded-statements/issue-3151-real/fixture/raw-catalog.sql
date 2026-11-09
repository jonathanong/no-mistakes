SELECT target.id, party.state
FROM notice_targets target
CROSS JOIN LATERAL (
  SELECT surface.kind AS state
  FROM placements surface
  WHERE surface.placement_id = target.placement_id
  UNION ALL
  SELECT recorded.state AS state
  FROM placement_activations recorded
  WHERE recorded.placement_id = target.placement_id
    AND recorded.placement_revision = target.activation_revision
  UNION ALL
  SELECT g.name AS state
  FROM groups g WHERE g.id = target.notice_id
  UNION ALL
  SELECT activation.state AS state
  FROM placements surface
  JOIN LATERAL (
    SELECT recorded.state, recorded.bound_by_admin
    FROM placement_activations recorded
    WHERE recorded.placement_id = surface.placement_id
      AND recorded.placement_revision = target.activation_revision
  ) activation ON true
  WHERE surface.placement_id = target.placement_id
    AND surface.kind IN ('comment', 'media')
    AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)
  UNION ALL
  SELECT activation.state AS state
  FROM placements surface
  LEFT JOIN LATERAL (
    SELECT recorded.state, recorded.bound_by_admin
    FROM placement_activations recorded
    WHERE recorded.placement_id = surface.placement_id
      AND recorded.placement_revision = target.activation_revision
  ) activation ON true
  JOIN placement_members member
    ON member.placement_id = surface.placement_id
    AND member.member_id = target.member_id
  WHERE surface.placement_id = target.placement_id
    AND surface.kind IN ('comment', 'media')
    AND TRUE AND (FALSE OR activation.bound_by_admin = FALSE)
) party
WHERE target.notice_id = $1
  AND target.id = ANY($2);
