import {query, sql} from '@example/db';
query('SELECT pg_sleep(1)');
query(sql`SELECT pg_catalog.pg_sleep_for(${duration})`);
query(sql`WITH timer AS (SELECT pg_sleep_until(${until}::timestamptz)) SELECT * FROM timer`);
query(sql`SELECT pg_sleep(GREATEST(EXTRACT(EPOCH FROM ${until}::timestamptz - clock_timestamp()), 0))`);
query(sql`SELECT ${unrelated}::text, pg_sleep(1)`);
query('SELECT pg_sleep FROM jobs AS pg_sleep_for');
