SELECT role FROM members m WHERE $2 <= m.user_id AND $2 >= m.user_id AND m.group_id = $1;
