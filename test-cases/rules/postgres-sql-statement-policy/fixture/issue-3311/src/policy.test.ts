import { query } from '@scope/psql';

query('CREATE TEMP TABLE embedded_table (id integer)');
query('ALTER TABLE embedded_table ADD COLUMN name text');
query('CREATE INDEX embedded_table_idx ON embedded_table (id)');
query('CREATE VIEW embedded_view AS SELECT id FROM embedded_table');
query('TRUNCATE embedded_table');
query('DROP INDEX embedded_table_idx');
query('DROP VIEW embedded_view');
query("SET statement_timeout = '1s'");
query("SELECT set_config('statement_timeout', '1s', false)");
