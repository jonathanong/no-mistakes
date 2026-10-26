import { query } from '@example/db';
import { createAccountsSql } from '../helpers/schema-sql';

query('CREATE TABLE direct_test (id integer)');
query('ALTER TABLE direct_test ADD CONSTRAINT valid_id CHECK (false)');
query('CREATE VIEW direct_view AS SELECT id FROM direct_test');
query('DROP VIEW direct_view');
query('CREATE MATERIALIZED VIEW snapshot AS SELECT id FROM direct_test');
query('ALTER SYSTEM SET statement_timeout = 100');
query('SET LOCAL session_replication_role = replica');
query('SELECT pg_sleep(1)');
query('SELECT PG_SLEEP(1)');
query('SELECT "PG_SLEEP"(1)');
query(createAccountsSql());
query('TRUNCATE direct_test');
query('INSERT INTO direct_test (id) VALUES (1)');
query('SELECT pg_get_indexdef(1)');
query('SELECT pg_sleepy(1)');

// SQL in an assertion is inert because it never reaches the configured executor.
const expected = 'DROP TABLE direct_test';
expect(expected).toContain('DROP TABLE');
