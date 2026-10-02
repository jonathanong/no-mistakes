SELECT row_to_json(
  -- no-mistakes-disable-next-line postgres-explicit-columns
  o.*
) FROM orders o;
