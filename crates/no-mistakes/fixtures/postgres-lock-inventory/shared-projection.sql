SELECT n.id FROM notices n
JOIN tasks t ON t.notice_id = n.id
JOIN owners o ON o.id = n.owner_id
FOR KEY SHARE OF n, t FOR SHARE OF o;
