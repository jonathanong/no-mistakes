import { pool } from '@example/db';
import { opaque } from '@opaque/helpers';
const { query } = pool;
opaque(pool);
const alias = query;
alias('DROP TABLE copied_before_receiver_escape');
