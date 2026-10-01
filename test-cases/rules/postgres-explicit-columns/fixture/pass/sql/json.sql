SELECT row_to_json(o.*) FROM orders o WHERE id = $1;
