import { pool } from '@db';
pool.query('SELECT id FROM users OFFSET 11');
pool.query('/* low/named */ SELECT id FROM users');
