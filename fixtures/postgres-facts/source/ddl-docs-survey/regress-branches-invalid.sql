-- Each line must remain an invalid statement; direct recognizer tests exercise its rejection path.
ALTER TABLE t ALTER SET (n_distinct = 1);
ALTER TABLE t ALTER c;
ALTER TYPE ADD ATTRIBUTE c integer;
ALTER TABLE t RESET (fillfactor,);
ALTER TABLE t SET (fillfactor = 70,);
DROP DOMAIN d1,;
CREATE TYPE t (input = f,);
CREATE TYPE t (= f);
CREATE MATERIALIZED VIEW m USING AS SELECT 1;
CREATE ROLE 123;
CREATE TABLE t (a,) AS SELECT 1;
ALTER AGGREGATE f(integer,) RENAME TO g;
ALTER AGGREGATE f(integer ORDER BY) RENAME TO g;
ALTER OPERATOR +(integer,) OWNER TO bob;
CREATE FUNCTION f RETURNS integer LANGUAGE sql AS 'SELECT 1';
CREATE FUNCTION f(a OUT integer RETURNS integer LANGUAGE sql AS 'SELECT 1';
CREATE TABLE t (a integer, b integer, FOREIGN KEY (a, b) REFERENCES p ON DELETE SET NULL (a,));
CREATE TABLE t (a integer, b integer, FOREIGN KEY (a, b) REFERENCES p ON DELETE SET NULL (a, b);
