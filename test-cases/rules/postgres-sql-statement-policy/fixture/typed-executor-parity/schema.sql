CREATE TABLE orders (
  id integer,
  doubled integer GENERATED ALWAYS AS (id * 2) STORED
);
