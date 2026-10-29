import sql, { type SQLStatement } from 'sql-template-strings'

export type Viewer = { id: string; admin: boolean }

export function buildPrivacyFilter(alias: string, viewer: Viewer | null): SQLStatement | null {
  if (viewer?.admin) return null
  if (!viewer) return sql`TRUE`
  return sql`(`.append(`${alias}.owner_id = `).append(sql`${viewer.id}`).append(')')
}
