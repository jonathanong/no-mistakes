SELECT (SELECT COUNT(*) FROM orders WHERE account_id = $1) > 5 AS is_frequent;
