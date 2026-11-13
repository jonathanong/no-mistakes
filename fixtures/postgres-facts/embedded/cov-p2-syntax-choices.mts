import { read, write, type QueryExecutor, type QueryOptions } from '@example/db'
declare const unknownOptions: unknown[]
declare const dynamicSql: string
function quiet(options: QueryOptions) { return options.timeout }
function forward(query: QueryExecutor) { query('SELECT 2') } // SQL:query('SELECT 2')=>SELECT 2
quiet() // QUIET
forward() // QUIET
read('SELECT 1', ...unknownOptions) // SQL:read('SELECT 1', ...unknownOptions)=>SELECT 1
quiet({ query: 1 }) // QUIET
quiet({ query: { value: read } }) // UNKNOWN:read
function fallbacks(options: QueryOptions, alternate: QueryExecutor, other: unknown) {
  ;(options?.query ?? alternate)('SELECT 3') // SQL:(options?.query ?? alternate)('SELECT 3')=>SELECT 3
  ;(options?.['query'] ?? alternate)(dynamicSql) // UNKNOWN:alternate
  ;(options.other ?? alternate)(dynamicSql) // UNKNOWN:alternate
  ;(other ?? alternate)(dynamicSql) // UNKNOWN:alternate
  const chosen = options.query ?? alternate
  opaque(chosen) // UNKNOWN:chosen
  const unused = options.query ?? alternate // QUIET
}
declare function opaque(value: unknown): void
read('SELECT 4', { query: () => write(dynamicSql) }) // SQL:read('SELECT 4', { query: () => write(dynamicSql) })=>SELECT 4
