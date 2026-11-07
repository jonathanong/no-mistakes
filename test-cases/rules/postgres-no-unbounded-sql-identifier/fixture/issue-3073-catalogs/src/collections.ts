import { read } from 'db'

const COLLECTIONS = [
  { key: 'a_count', relation: { table: 'rel_a', direction: 'object' } },
] as const

const SELECT_SQL = COLLECTIONS.filter(c => c.relation)
  .map(c =>
    `(SELECT COUNT(*) FROM ${c.relation.table} r
       WHERE r.${c.relation.direction === 'subject'
         ? 'object_id' : 'subject_id'} = $1) AS ${c.key}`)
  .join(',\n')

export const counts = (id: string) =>
  read(`/* counts */ SELECT ${SELECT_SQL}`, [id])
