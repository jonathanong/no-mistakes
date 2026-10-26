import { query } from '@example/db';
const sql = 'SELECT 1 OFFSET 3';
// no-mistakes-disable-next-line postgres-no-offset: executor directive owns constant findings
query(sql);
