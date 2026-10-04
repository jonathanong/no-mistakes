-- A statement runner that continues after a rejected list bound keeps the child alive.
CREATE TABLE list_child(id integer);
CREATE TEMP TABLE list_parent(id integer) PARTITION BY LIST (id);
CREATE TEMP TABLE list_child(id integer);
ALTER TABLE list_parent ATTACH PARTITION list_child FOR VALUES IN (,);
DROP TABLE list_parent;
SELECT * FROM list_child;
-- A missing hash modulus likewise leaves its child independent.
CREATE TABLE hash_child(id integer);
CREATE TEMP TABLE hash_parent(id integer) PARTITION BY HASH (id);
CREATE TEMP TABLE hash_child(id integer);
ALTER TABLE hash_parent ATTACH PARTITION hash_child FOR VALUES WITH (REMAINDER 0);
DROP TABLE hash_parent;
SELECT * FROM hash_child;
