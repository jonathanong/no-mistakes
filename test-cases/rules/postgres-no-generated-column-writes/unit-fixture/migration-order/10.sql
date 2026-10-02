DROP TABLE orders;
CREATE TABLE orders (id int, generated int GENERATED ALWAYS AS (id + 1) STORED);
