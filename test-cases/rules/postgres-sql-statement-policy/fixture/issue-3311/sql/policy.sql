CREATE TEMP TABLE default_table (id integer);
ALTER TABLE default_table ADD COLUMN name text;
CREATE INDEX default_table_idx ON default_table (id);
CREATE VIEW default_view AS SELECT id FROM default_table;
TRUNCATE default_table;
DROP INDEX default_table_idx;
DROP VIEW default_view;
SET statement_timeout = '1s';
SELECT set_config('statement_timeout', '1s', false);
