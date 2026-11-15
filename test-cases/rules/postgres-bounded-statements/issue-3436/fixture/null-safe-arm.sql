DELETE FROM invites
WHERE community_id = $1
  AND accepted_at IS NULL
  AND declined_at IS NULL
  AND revoked_at IS NULL
  AND (
    invited_user_id IS NOT DISTINCT FROM $2
    OR invited_email IN (
      SELECT email FROM emails WHERE user_id = $2 LIMIT 10
    )
  );
