import { pool } from '@example/db';

// Copying after an override must stay visible, without claiming the imported method survived.
pool.query = replacement;
const { query: changed } = pool;
const changedAlias = changed;
changedAlias('DROP TABLE replaced_receiver');
