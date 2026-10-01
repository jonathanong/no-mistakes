UPDATE orders SET status = 'paid' WHERE id = $1 RETURNING *;
