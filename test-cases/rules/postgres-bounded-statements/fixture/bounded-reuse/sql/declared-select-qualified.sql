SELECT g.id, public.fn_remaining(g.id) AS target
FROM grants g
WHERE g.user_id = $1
ORDER BY g.id DESC
LIMIT $2
