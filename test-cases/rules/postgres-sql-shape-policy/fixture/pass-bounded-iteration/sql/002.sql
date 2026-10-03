SELECT id FROM orders WHERE id >= $1 AND id < $2 ORDER BY id LIMIT $3;
SELECT id FROM orders WHERE $1 < id AND $2 >= id ORDER BY id DESC LIMIT $3;
