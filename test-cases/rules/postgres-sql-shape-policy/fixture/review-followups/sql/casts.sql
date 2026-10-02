SELECT COUNT(*)::integer > 0 FROM orders;
SELECT (SELECT COUNT(*)::integer FROM orders)::bigint = 0;
