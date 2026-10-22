import { pool } from '@db';
// Intentionally unreachable: all potential exporters remain conservative providers.
export const unused = true;
pool.query('SELECT id FROM users OFFSET 8');
