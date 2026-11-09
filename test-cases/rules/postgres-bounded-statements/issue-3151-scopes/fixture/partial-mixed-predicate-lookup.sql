SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members_mixed_case m
  WHERE m.group_id = $1 AND m.user_id >= $2 AND m.user_id <= $2
    AND m."Removed_At" IS NULL), false);
