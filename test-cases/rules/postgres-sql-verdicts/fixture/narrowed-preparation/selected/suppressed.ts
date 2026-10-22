import { pool } from '@db';
// no-mistakes-disable-next-line postgres-no-offset
pool.query('SELECT id FROM users OFFSET 4');
