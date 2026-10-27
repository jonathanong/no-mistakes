UPDATE exports SET s3_key = NULL WHERE id = sql_placeholder_1 AND expires_at IS NOT NULL;
