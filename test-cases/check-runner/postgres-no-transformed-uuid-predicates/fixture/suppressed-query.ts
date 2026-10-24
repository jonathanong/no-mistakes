// no-mistakes-disable-file postgres-no-transformed-uuid-predicates
import { query } from '@db';
query('SELECT id FROM users WHERE id::text = $1', []);
