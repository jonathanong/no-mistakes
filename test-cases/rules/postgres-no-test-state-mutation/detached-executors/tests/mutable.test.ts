import { pool } from '@example/db';
let mutable = pool.query.bind(pool);
mutable = replacement;
const first = mutable;
const second = first;
second('DROP TABLE mutable_executor');
