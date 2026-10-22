import { pool, sql } from '@db';
import { client } from '@other';
// Both configured executors must remain safe consumers of this tagged const.
const shared = sql`/* low/shared */ SELECT id FROM users OFFSET 13`;
pool.query(shared);
client.query(shared);
