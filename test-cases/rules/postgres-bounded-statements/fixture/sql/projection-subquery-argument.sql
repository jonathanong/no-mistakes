-- Other SQL dialects represent a direct subquery function argument separately from a list.
SELECT generate_series(SELECT id FROM orders);
