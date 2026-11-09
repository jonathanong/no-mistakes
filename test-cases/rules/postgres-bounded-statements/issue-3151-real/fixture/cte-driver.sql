WITH local_target AS (
  SELECT target.id, target.placement_id, target.activation_revision
  FROM notice_targets target
)
SELECT local_target.id, activation.state
FROM local_target
CROSS JOIN LATERAL (
  SELECT recorded.state
  FROM placement_activations recorded
  WHERE recorded.placement_id = local_target.placement_id
    AND recorded.placement_revision = local_target.activation_revision
) activation;
