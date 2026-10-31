SELECT member_id FROM memberships WHERE account_id = $1;
SELECT member_id FROM memberships WHERE member_id = $1;
SELECT member_id FROM memberships WHERE account_id = $1 AND member_id = $2;
