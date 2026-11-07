import { read } from '@bench/db';
const TABLE = { sql: '/* table/gap */ SELECT 1' } as const;
eval('void 0');
read(TABLE.sql); // low: false
