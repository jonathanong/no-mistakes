SELECT role FROM members WHERE group_id = 1 AND user_id >= random() AND user_id <= random();
