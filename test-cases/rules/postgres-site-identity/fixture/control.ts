import { query } from 'db';
query('SELECT 1 FROM accounts WHERE id = $1');
query('SELECT 1 FROM accounts WHERE fn(id, token) = $1');
