import { write, sql, QueryOptions } from '@example/db';
export default async function (options: QueryOptions) {
  await write(sql`SELECT id FROM users WHERE id = 1 FOR UPDATE`, options);
}
