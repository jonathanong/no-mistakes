import { pool } from '@db';
pool.query('SELECT id FROM users OFFSET 9');
