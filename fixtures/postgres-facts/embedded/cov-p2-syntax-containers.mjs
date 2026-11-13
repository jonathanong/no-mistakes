import { read } from '@example/db'
export function containers(dynamicSql) {
const keyed = { [null]: read }
keyed.null(dynamicSql) // UNKNOWN:keyed.null
const rows = [read, ...[]]
rows[0](dynamicSql) // UNKNOWN:rows
const bag = Object.freeze({ query: read })
bag?.['query'](dynamicSql) // UNKNOWN:bag?.['query']
}
read('SELECT 4') // SQL:read('SELECT 4')=>SELECT 4
