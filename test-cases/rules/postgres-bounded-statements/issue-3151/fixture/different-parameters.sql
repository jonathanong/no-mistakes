SELECT role FROM members WHERE group_id = $1 AND user_id >= $2 AND user_id <= $3;
