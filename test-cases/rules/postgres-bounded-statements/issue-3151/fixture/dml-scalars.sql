UPDATE members SET role = (SELECT role FROM members) WHERE group_id = 1 AND user_id = 1;
DELETE FROM members WHERE group_id = 1 AND user_id = 1 AND role = (SELECT role FROM members);
