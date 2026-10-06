import { query } from '@example/db';
query('CREATE DATABASE scratch TEMPLATE template0');
query('SET LOCAL session_replication_role = replica');
query("SELECT set_config('session_replication_role', 'replica', true)");
query('ALTER SYSTEM SET statement_timeout = 100');
query('CREATE OR REPLACE PROCEDURE reset_orders() LANGUAGE plpgsql AS $$ BEGIN NULL; END $$');
