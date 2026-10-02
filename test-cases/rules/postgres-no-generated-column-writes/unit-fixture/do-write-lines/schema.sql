CREATE TABLE orders (id int, computed int GENERATED ALWAYS AS (id + 1) STORED);
