SELECT date_trunc('day', created_at) AS d, COUNT(*) FROM orders GROUP BY d;
