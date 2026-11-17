import { write, sql, QueryOptions } from '@example/db';
export async function lockMember(cid: string, uid: string, options: QueryOptions) {
  await write(sql`SELECT id FROM users WHERE company_id = ${cid} AND id = ${uid} FOR UPDATE`, options);
}
export const lockArrow = async (options: QueryOptions) => {
  await write(sql`SELECT id FROM users WHERE id = 1 FOR UPDATE`, options);
};
async function unreferenced(options: QueryOptions) {
  await write(sql`SELECT id FROM users WHERE id = 2 FOR UPDATE`, options);
}
