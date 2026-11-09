SELECT COALESCE((SELECT m.role IN ('owner','moderator') FROM members_mixed_key m
  WHERE m.group_id = $1 AND m."User_Id" >= $2 AND m."User_Id" <= $2
    AND m.removed_at IS NULL), false);
