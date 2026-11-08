import sql from 'sql-template-strings';
import { read } from '@example/db';
import { pageClause } from '@helpers/barrel';
export async function page() {
  const query = sql`SELECT id FROM accounts `;
  query.append(pageClause());
  return read(query);
}
