import { read } from '@example/db';

declare function unrelated(value: unknown): unknown;
const local = { sql: 'SELECT 1 OFFSET 8' };

// A second parsed file gets its own opaque model pool.
unrelated(local);
unrelated({ nested: [local] });
read('SELECT 1 OFFSET 9');
