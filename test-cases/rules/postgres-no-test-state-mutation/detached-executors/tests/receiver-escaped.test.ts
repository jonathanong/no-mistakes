import { pool } from '@example/db';
import { opaque } from '@opaque/helpers';
opaque(pool);
const { query } = pool;
const alias = query;
alias('DROP TABLE escaped_receiver');
