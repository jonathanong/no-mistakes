SELECT id, order_index FROM questions WHERE group_id = $1 AND deleted_at IS NULL ORDER BY order_index;
SELECT id, order_index FROM questions WHERE group_id = $1 ORDER BY order_index;
