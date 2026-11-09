SELECT role FROM members m WHERE m.group_id = $1 AND (m.user_id >= ($2)) AND ((m.user_id) <= $2);
