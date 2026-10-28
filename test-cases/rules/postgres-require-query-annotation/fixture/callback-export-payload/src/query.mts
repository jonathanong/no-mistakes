import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { page } from './helpers.mts'
import { mutate } from './mutator.mts'

export const direct = sql`/* exported/direct */ SELECT 1`
const aliased = sql`/* exported/alias */ SELECT 1`
export { aliased as publicAlias }
const defaulted = sql`/* exported/default */ SELECT 1`
export default defaulted

// The cyclic importer mutates exported objects without adding local references.
mutate()
page(direct, statement => read(statement)) // low: false
page(aliased, statement => read(statement)) // low: false
page(defaulted, statement => read(statement)) // low: false

const privateBase = sql`/* private */ SELECT 1`
page(privateBase, statement => read(statement)) // low: true

function privateShadow() {
  // This exact binding is private despite the unrelated module export's name.
  const direct = sql`/* private/shadow */ SELECT 1`
  page(direct, statement => read(statement)) // low: true
}
