import sql from 'sql-template-strings'
import { read } from 'db'
import { selectWithOwner } from './select-with-owner.ts'

export async function getItem(byId: boolean, id: string, slug: string) {
  const query = byId
    ? selectWithOwner(sql`/* getItemById */ SELECT `).append(sql` WHERE c.id = ${id}`)
    : selectWithOwner(sql`/* getItemBySlug */ SELECT `).append(sql` WHERE c.slug = ${slug}`)
  query.append(sql` AND c.deleted_at IS NULL LIMIT 1`)
  return read(query.text, query.values)
}
