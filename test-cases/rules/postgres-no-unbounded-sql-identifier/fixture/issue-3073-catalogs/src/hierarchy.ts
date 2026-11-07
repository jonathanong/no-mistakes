import { read } from 'db'
import { getRelationTableNameOrThrow } from './barrel.ts'

const PARENT_TABLE = getRelationTableNameOrThrow({
  subject: 'topic', predicate: 'parent', object: 'topic',
})

export const listParents = (id: string) =>
  read(
    `/* listParents */ SELECT object_id FROM ${PARENT_TABLE}
       WHERE subject_id = $1 AND deleted_at IS NULL`,
    [id],
  )

export function localParents(id: string) {
  const table = getRelationTableNameOrThrow({
    subject: 'topic', predicate: 'parent', object: 'topic',
  })
  return read(`/* localParents */ SELECT object_id FROM ${table} WHERE subject_id = $1`, [id])
}
