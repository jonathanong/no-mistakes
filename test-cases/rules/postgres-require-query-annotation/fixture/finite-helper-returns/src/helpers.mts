import sql from 'sql-template-strings'

export function named(flag: boolean) {
  if (flag) return sql`/* importedOne */ SELECT 1`
  return sql`/* importedTwo */ SELECT 2`
}
export function missing(flag: boolean) {
  if (flag) return sql`SELECT 1`
  return sql`SELECT 2`
}
export function mixed(flag: boolean) {
  if (flag) return sql`/* importedMixed */ SELECT 1`
  return sql`SELECT 2`
}
function abrupt(_value: never): never { throw Error('invalid') }
export function switched(value: 'a' | 'b') {
  switch (value) {
    case 'a': return sql`/* importedA */ SELECT 1`
    case 'b': return sql`/* importedB */ SELECT 2`
    default: return abrupt(value)
  }
}
