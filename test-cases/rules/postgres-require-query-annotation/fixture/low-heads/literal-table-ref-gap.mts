import { read } from '@bench/db';
const TABLE = { sql: '/* table/ref-gap */ SELECT 1' } as const;
function run() {
  eval('void 0');
  read(TABLE.sql); // low: false
}
