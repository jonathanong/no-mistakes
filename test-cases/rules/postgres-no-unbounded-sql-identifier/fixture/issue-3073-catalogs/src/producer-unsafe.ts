import { read } from 'db'
import { getRelationTableNameOrThrow } from './barrel.ts'
export function changedProducer(id: string) {
  const table = getRelationTableNameOrThrow({ subject: 'topic', predicate: 'parent', object: 'topic' })
    .replace('rel_topic_parent_topic', process.env.UNCHECKED_TABLE)
  return read(`SELECT object_id FROM ${table} WHERE subject_id = $1`, [id])
}
