SELECT p.user_id FROM participants p
WHERE p.conversation_id = $1 AND p.user_id = $2;

SELECT p.user_id FROM participants p
WHERE p.conversation_id = $1 OR p.user_id = $2;

SELECT p.user_id FROM participants p WHERE p.conversation_id = $1;

SELECT DISTINCT ON (i.id) i.id FROM items i
WHERE i.url_id = $1 AND i.deleted_at IS NULL
ORDER BY i.id;

SELECT DISTINCT ON (a.url_id) a.id FROM items a
WHERE a.deleted_at IS NULL
ORDER BY a.url_id, a.id DESC;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id IN ($1, $2) AND i.deleted_at IS NULL
ORDER BY i.url_id, i.id DESC;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id = ANY($1) AND i.deleted_at IS NULL
ORDER BY i.url_id, i.id DESC;

SELECT DISTINCT ON (i.url_id) i.id, generate_series(1, 2)
FROM items i
WHERE i.url_id = $1 AND i.deleted_at IS NULL
ORDER BY i.url_id, i.id DESC;
