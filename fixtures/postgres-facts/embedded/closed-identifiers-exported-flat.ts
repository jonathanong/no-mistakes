import { query } from '@example/db';
export const tables = { users: 'app.users', posts: 'app.posts' } as const;
const alias = tables;
query(`SELECT id FROM ${tables.users} WHERE active`);
query(`SELECT id FROM ${alias.posts} WHERE active`);
export const nested = { user: { table: 'app.users' } } as const;
query(`SELECT id FROM ${nested.user.table} WHERE active`);
