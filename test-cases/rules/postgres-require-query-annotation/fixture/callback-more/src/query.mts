import sql from 'sql-template-strings'
import { read, write } from '@bench/db'
import { page, mixed } from './helpers.mts'
import { unavailable } from 'missing-callback-package'

function appendOnly(statement: unknown, value: unknown) {
  statement.append(sql` AND id = ${value}`)
}
function unknownEffect(statement: unknown) {
  statement.text = 'SELECT 1'
}
function queryLocal(execute: (statement: unknown) => Promise<unknown>) {
  return execute(sql`/* localCallback */ SELECT 1`.append(unknownSuffix))
}
function queryUntrusted(execute: (statement: unknown) => Promise<unknown>) {
  return execute(customTag`/* untrusted */ SELECT 1`)
}
let changed = queryLocal
changed = queryUntrusted
const options = { timeout: 1000 }

const base = sql`/* asyncCallback */ SELECT 1`
appendOnly(base, 1)
page(base, async statement => {
  statement.append(sql` ORDER BY id`)
  appendOnly(statement, 2)
  const result = await read(statement) // low: true
  return result
})

queryLocal(statement => write(statement, { ...options, readOnly: false })) // low: true
queryUntrusted(statement => write(statement, { ...options, readOnly: false })) // low: false
changed(statement => write(statement, { ...options, readOnly: false })) // low: false

const badBase = sql`/* badAsync */ SELECT 1`
page(badBase, async statement => {
  unknownEffect(statement)
  return read(statement) // low: false
})

const mixedBase = sql`/* mixedAsync */ SELECT 1`
mixed(mixedBase, async statement => read(statement)) // low: false

queryLocal(statement => {
  unknownEffect(statement)
  return write(statement, { ...options, readOnly: false }) // low: false
})
queryLocal(statement => write?.(statement, { ...options, readOnly: false })) // low: false
queryLocal(statement => write(statement, { get readOnly() { return false } })) // low: false
queryLocal(statement => {
  const alias = statement
  return write(alias, { readOnly: false }) // low: false
})
queryLocal(statement => {
  statement = sql`SELECT 1`
  return write(statement, { readOnly: false }) // low: false
})
unavailable(sql`/* noSource */ SELECT 1`, async statement => read(statement)) // low: false
