-- PostgreSQL accepts either hash-option order. The attach must remove the
-- temporary child when its temporary parent is dropped, exposing the physical namesake.
CREATE TABLE child(id integer);
CREATE TEMP TABLE parent(id integer) PARTITION BY HASH(id);
CREATE TEMP TABLE child(id integer);
ALTER TABLE parent ATTACH PARTITION child FOR VALUES WITH (REMAINDER 0, MODULUS 4);
DROP TABLE parent;
SELECT * FROM child;
