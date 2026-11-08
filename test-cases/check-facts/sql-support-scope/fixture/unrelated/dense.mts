import sql from 'sql-template-strings';
import { read } from '@example/db';
export async function unrelated() {
  const query = sql`SELECT id FROM ignored WHERE ignored = ${true} `;
  query.append(sql`OFFSET 30`);
  return read(query);
}
