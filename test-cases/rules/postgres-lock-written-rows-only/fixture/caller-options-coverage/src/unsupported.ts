import { write, sql, beginTransaction } from '@example/db';
async function duplicate(left: unknown, right: unknown) {
  await write('SELECT id FROM items FOR UPDATE', left, right);
}
async function negated(flag: boolean, opts: unknown) {
  await write(!flag ? sql`SELECT id FROM items FOR UPDATE` : sql`SELECT id FROM items`, opts);
}
async function dynamic(statement: string, opts: unknown) {
  await write(statement, opts);
}
async function mixed(id: string, opts: unknown) {
  await write(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`, [id], opts);
}
async function calculated(id: string, opts: unknown) {
  await write(sql`SELECT id FROM items WHERE id = ${id.toString()} FOR UPDATE`, opts);
}
async function run(id: string) {
  using tx = beginTransaction();
  await duplicate({ query: tx }, { query: tx });
  await negated(false, { query: tx });
  await dynamic('SELECT id FROM items FOR UPDATE', { query: tx });
  await mixed(id, { query: tx });
  await calculated(id, { query: tx });
}
