import { pool } from '@example/db';
import { opaque } from '@opaque/helpers';
let changing = pool;
opaque(changing);
changing = pool.query;
// Replacing an alias does not undo the earlier receiver escape.
const { query } = pool;
query('DROP TABLE escaped_receiver_history');
