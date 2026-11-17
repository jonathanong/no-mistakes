import { write, sql, beginTransaction } from '@example/db';
async function lockMember(id: string, options: { query?: unknown }) {
  await write(sql`SELECT id FROM users WHERE id = ${id} FOR UPDATE`, options);
}
export async function run(id: string) {
  await using first = await beginTransaction();
  await lockMember(id, { query: first });
  await using second = await beginTransaction();
  await lockMember(id, { query: second });
  await using third = await beginTransaction();
  await lockMember(id, { query: third });
}
