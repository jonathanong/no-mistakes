import { pool } from '@example/db';
const { query } = pool;
retain(query);
const alias = query;
alias('DROP TABLE escaped_executor');
