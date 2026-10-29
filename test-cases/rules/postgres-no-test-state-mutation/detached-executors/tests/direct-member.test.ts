import { pool } from '@example/db';
pool.query('DROP TABLE direct_member_control');
const receiver = pool;
receiver.query('DROP TABLE receiver_member_control');
