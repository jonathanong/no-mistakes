SELECT item.id, stream_view.id AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id AND stream.deleted_at IS NULL
LEFT JOIN stream_view ON stream_view.id = stream.id AND stream_view.is_enabled = TRUE
WHERE item_type = $1 AND item.deleted_at IS NULL
ORDER BY item.position, item.id;

SELECT item.id, stream_view.id AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id AND stream.deleted_at IS NULL
LEFT JOIN stream_view ON stream_view.id = stream.id AND stream_view.is_enabled = TRUE
WHERE item.item_type = $1 AND item.deleted_at IS NULL
ORDER BY item.position, item.id;

SELECT item.id FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id AND stream.deleted_at IS NULL
WHERE item_type = $1 AND item.deleted_at IS NULL;

SELECT item.id, view_row.id AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id AND stream.deleted_at IS NULL
LEFT JOIN ambiguous_stream_view view_row ON view_row.id = stream.id
WHERE item_type = $1 AND item.deleted_at IS NULL;

SELECT item.id, view_row.id AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id
LEFT JOIN grouped_stream_view view_row ON view_row.id = stream.id
WHERE item_type = $1;

SELECT item.id, view_row.id AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id
LEFT JOIN wildcard_stream_view view_row ON view_row.id = stream.id
WHERE item_type = $1;

SELECT item.id, view_row.item_type AS view_id
FROM content_items item
LEFT JOIN streams stream ON stream.id = item.stream_id AND stream.deleted_at IS NULL
LEFT JOIN stream_view AS view_row(item_type, is_enabled) ON view_row.item_type = stream.id
WHERE item_type = $1 AND item.deleted_at IS NULL;
