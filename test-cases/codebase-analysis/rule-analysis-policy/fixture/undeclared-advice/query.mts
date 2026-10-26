import type { PoolClient } from '@example/db';

declare const context: { db: PoolClient };
context.db.query('SELECT 1');
