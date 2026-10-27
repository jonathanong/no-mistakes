SELECT row_to_json(
  -- postgres-explicit-columns
  o.*
) FROM orders o;
