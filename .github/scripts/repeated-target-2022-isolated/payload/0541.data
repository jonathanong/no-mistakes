import sql from 'sql-template-strings'
import { read } from '@example/db'

export function list() {
  const query = sql`/* opaque initializer */ SELECT 1`
  const ignored = eval("query.append(' UPDATE posts SET title = title')")
  void ignored
  return read(query)
}
