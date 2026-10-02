CREATE TABLE orders (id bigint, old_gen bigint GENERATED ALWAYS AS (id + 1) STORED);
ALTER TABLE orders ADD COLUMN new_gen bigint GENERATED ALWAYS AS (id + 2) STORED;
