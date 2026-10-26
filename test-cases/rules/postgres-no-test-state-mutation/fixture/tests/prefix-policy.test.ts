import { query } from '@example/db';

declare const opaque: string[];

// Each complete statement precedes an opaque append; its policy fact is still certain.
query('CREATE TABLE prefix_table (id integer); '.append(...opaque));
query('SET session_replication_role = replica; '.append(...opaque));
query('SELECT pg_sleep(1); '.append(...opaque));

// The unknown remainder can still contain a mutation even with a safe prefix.
query('SELECT 1; '.append(...opaque));
