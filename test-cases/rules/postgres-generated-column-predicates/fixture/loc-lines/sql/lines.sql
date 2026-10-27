SELECT id
FROM orders
WHERE created_at <> $1
ORDER BY created_at;
