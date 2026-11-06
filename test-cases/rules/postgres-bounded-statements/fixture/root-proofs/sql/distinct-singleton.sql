SELECT DISTINCT ON (i.url_id) i.id FROM items i ORDER BY i.url_id;

SELECT DISTINCT ON (i.url_id, i.id) i.id FROM items i
WHERE i.url_id = $1 ORDER BY i.url_id, i.id;

SELECT DISTINCT ON (i.url_id, i.deleted_at) i.id FROM items i
WHERE i.url_id = $1 AND i.deleted_at = $2 ORDER BY i.url_id, i.deleted_at;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE $1 = i.url_id ORDER BY i.url_id;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id = id ORDER BY i.url_id;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id = $1::uuid ORDER BY i.url_id;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id = gen_random_uuid() ORDER BY i.url_id;

SELECT DISTINCT ON (i.url_id) i.id FROM items i
WHERE i.url_id = $1 OR i.url_id = $2 ORDER BY i.url_id;
