-- DDL and DML share a line: the INSERT runs after the CREATE, so it is checked.
CREATE TABLE t (id int, g int GENERATED ALWAYS AS (id) STORED); INSERT INTO t (g) VALUES (1);
