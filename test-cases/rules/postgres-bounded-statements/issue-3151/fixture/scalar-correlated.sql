SELECT (SELECT inner_m.role FROM members inner_m WHERE inner_m.group_id = m.group_id AND inner_m.user_id = m.user_id) FROM members m WHERE m.group_id = $1 AND m.user_id = $2;
