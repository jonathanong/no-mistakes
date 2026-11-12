import { query } from '@scope/psql';
query("SET statement_timeout = '1s'");
query('TRUNCATE TABLE things');
