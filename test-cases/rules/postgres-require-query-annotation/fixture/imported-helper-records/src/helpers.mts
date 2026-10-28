import sql from 'sql-template-strings'

export function named(value: unknown) {
  return { text: `/* importedPlain */ SELECT ${value}`, values: [value] }
}
export function missing() {
  return { text: 'SELECT 1', values: [] }
}
export function builder(value: unknown) {
  return sql`/* importedBuilder */ SELECT ${value}`
}
export function mutable() {
  return { text: '/* mutableHelper */ SELECT 1', values: [] }
}
mutable = missing
