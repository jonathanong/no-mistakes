SELECT ROW_TO_JSON(o.*) FROM orders o WHERE id = $1;
