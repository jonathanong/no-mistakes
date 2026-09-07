SELECT id FROM topics WHERE parent_id IS NOT NULL AND id = $1;
