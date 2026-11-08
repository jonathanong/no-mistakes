import sql from 'sql-template-strings';
import { read } from '@benchmark/db';
import { pageClause } from '@sql/barrel';
export async function fetchPage() {
  const query = sql`SELECT 1 `;
  query.append(pageClause());
  return read(query);
}
