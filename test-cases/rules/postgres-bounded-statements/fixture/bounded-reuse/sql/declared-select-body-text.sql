SELECT g.id, fn_body_text(g.id) AS target
FROM grants g
WHERE g.user_id = $1
ORDER BY g.id DESC
LIMIT $2
