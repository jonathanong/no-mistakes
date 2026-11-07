import sql, { type SQLStatement } from 'sql-template-strings'
import { read } from 'db'
const SELECT_COLUMNS = sql`a.id, a.status` // module-const fragment
function buildRead(ids: string[]): SQLStatement {
  return sql`id, url FROM items WHERE id = ANY(${ids}::uuid[])`
}
export async function getOne(id: string, flag: 'x' | 'y' | null) {
  const statement = sql``.append(SELECT_COLUMNS).append(sql` FROM appeals a WHERE a.id = ${id}`)
  if (flag) statement.append(sql` AND reason = 'ring'`)
  statement.append(sql` ORDER BY a.id DESC LIMIT ${10}`)
  return read(`/* getOne */\n SELECT ${statement.text}`, statement.values)
}
export async function getMany(ids: string[]) {
  const s = buildRead(ids)
  return read(`/* getMany */ SELECT ${s.text}`, s.values)
}
