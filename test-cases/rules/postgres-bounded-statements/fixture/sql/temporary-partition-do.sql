-- DO wrappers must recover nested DETACH and ATTACH transitions in source order.
CREATE TABLE orders(id integer);
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
CREATE TEMP TABLE orders PARTITION OF accounts FOR VALUES FROM (0) TO (10);
DO $$
BEGIN
  IF true THEN
    ALTER TABLE accounts DETACH PARTITION orders;
  END IF;
END
$$ LANGUAGE plpgsql;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
CREATE TEMP TABLE accounts(id integer) PARTITION BY RANGE (id);
DO $$
BEGIN
  IF true THEN
    ALTER TABLE accounts ATTACH PARTITION orders FOR VALUES FROM (0) TO (10);
  END IF;
END
$$;
DROP TABLE accounts CASCADE;
SELECT * FROM orders;
