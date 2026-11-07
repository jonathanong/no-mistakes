declare function unknownRows(...args: unknown[]): any
declare const predicate: (row: unknown) => boolean
const project = row => row.relation.table

const ROWS = [{ key: 'a_count', relation: { table: 'rel_a', direction: 'subject' } }] as const
export const GOOD = ROWS.filter(row => !!row.relation)
  .map(row => row.key === 'a_count' ? row['relation']['table'] + ' AS a_count' : 'rel_b')
  .join(',')

const BAD_ROWS = [{ key: 'a_count', relation: { table: 'rel_a' } }] as const
export const INVALID_ARGUMENTS = BAD_ROWS.map(row => row.relation.table, null).join(',')
export const NON_MEMBER = unknownRows(BAD_ROWS).join(',')
export const UNSUPPORTED = BAD_ROWS.slice(1).join(',')
export const NAMED_CALLBACK = BAD_ROWS.map(project).join(',')
export const BLOCK_CALLBACK = BAD_ROWS.map(row => { return row.relation.table }).join(',')
export const MISSING_FIELD = BAD_ROWS.map(row => row.missing).join(',')
export const FILTER_JOIN = BAD_ROWS.filter(row => row.relation).join()
export const UNKNOWN_CALLBACK = BAD_ROWS.filter(predicate).join(',')
export const BOUND = BAD_ROWS.map.call(BAD_ROWS, row => row.relation.table).join(',')
export const NO_ARGUMENTS = BAD_ROWS.map().join(',')
export const LITERAL_JOIN = BAD_ROWS.join(',')
export const OPAQUE_CALL = unknownRows(BAD_ROWS)

let mutableRows = unknownRows()
export const MUTABLE = mutableRows.map(row => row.relation.table).join(',')
const cyclicRows = cyclicRows
export const CYCLIC = cyclicRows.map(row => row.relation.table).join(',')

const STRING_ROWS = [{ key: 'a_count', relation: { table: 'rel_a' } }] as const
export const ALREADY_STRINGS = STRING_ROWS.map(row => row.relation.table).map(value => value.length).join(',')
