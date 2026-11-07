import { read } from 'db'

const MUTATING = [{ key: 'a_count', relation: { table: 'rel_a' } }] as const
const MUTATING_SQL = MUTATING.map(c => {
  ;(c.relation as { table: string }).table = process.env.TABLE!
  return `SELECT COUNT(*) FROM ${c.relation.table}`
}).join(',')
export const mutatingCounts = () => read(`/* mutatingCounts */ SELECT (${MUTATING_SQL})`)

const RAW_FILTER = [{ key: 'a_count', relation: { table: 'rel_a' } }] as const
const rows = RAW_FILTER.filter(c => c.relation)
;(rows[0].relation as { table: string }).table = process.env.TABLE!
const FILTER_SQL = RAW_FILTER.map(c => `(SELECT COUNT(*) FROM ${c.relation.table}) AS ${c.key}`).join(',')
export const filteredCounts = () => read(`/* filteredCounts */ SELECT ${FILTER_SQL}`)

const OPEN_COLLECTION = [{ key: 'a_count', relation: { table: process.env.TABLE } }] as const
const OPEN_SQL = OPEN_COLLECTION.map(c => `(SELECT COUNT(*) FROM ${c.relation.table}) AS ${c.key}`).join(',')
export const openCounts = () => read(`/* openCounts */ SELECT ${OPEN_SQL}`)
