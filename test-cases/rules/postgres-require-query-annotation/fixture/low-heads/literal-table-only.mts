import { read } from '@bench/db';
const TABLE = { sql: '/* table/only */ SELECT 1' } as const;
read(TABLE.sql); // low: true
