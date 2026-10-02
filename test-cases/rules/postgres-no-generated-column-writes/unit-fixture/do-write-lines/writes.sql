-- Keep inner writes at distinct physical lines, including after wrapper recovery.
DO $$
BEGIN
  -- no-mistakes-disable-next-line postgres-no-generated-column-writes
  INSERT INTO orders (computed) VALUES (1);
  UPDATE orders SET computed = 2; -- no-mistakes-disable-line postgres-no-generated-column-writes
  INSERT INTO orders (computed) VALUES (3);
  UPDATE orders SET computed = 4;
END
$$;
