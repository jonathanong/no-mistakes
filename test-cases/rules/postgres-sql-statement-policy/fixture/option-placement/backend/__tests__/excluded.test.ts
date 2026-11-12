import { query } from '@scope/psql';
query("SET statement_timeout = '2s'");
query('TRUNCATE TABLE excluded_things');
