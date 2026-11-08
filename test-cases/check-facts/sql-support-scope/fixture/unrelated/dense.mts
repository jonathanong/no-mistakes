import sql from 'sql-template-strings';
import { read } from '@example/db';
export async function unrelated() {
  const query = sql`SELECT id FROM ignored OFFSET 30`;
  query.append(sql`WHERE ignored = ${true}`);
  return read(query);
}
