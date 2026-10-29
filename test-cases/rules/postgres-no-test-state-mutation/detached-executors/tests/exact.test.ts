import { pool, query as direct } from '@example/db';

const { query } = pool;
query('DROP TABLE detached_capture');
const first = query;
const second = first;
second('TRUNCATE detached_alias');

const bound = pool.query.bind(pool);
bound('DROP TABLE detached_bound');
const boundAlias = bound;
boundAlias('ALTER TABLE detached_bound ADD COLUMN value integer');

direct('DROP TABLE direct_control');

const method = pool.query;
method('DROP TABLE copied_member');
const rebound = method.bind(pool);
rebound('DROP VIEW rebound_method');
