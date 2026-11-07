import sql, { SQL as namedSql } from 'sql-template-strings'
import * as namespaceSql from 'sql-template-strings'

function builder() { return sql`id FROM items` }
function recursiveBuilder(): unknown { return recursiveBuilder() }
async function asyncBuilder() { return sql`id FROM items` }

export function boundaries(factory: () => unknown) {
  const chainedStatement = sql`id FROM items`
  chainedStatement.append(sql` WHERE active = ${true}`).append(sql` LIMIT ${10}`)
  const namedStatement = namedSql`id FROM items`
  namedStatement.append(namedSql` LIMIT ${10}`)
  const returnedStatement = builder()
  const textReceiver = sql`id FROM items`
  textReceiver.text.append(sql` LIMIT ${10}`)
  const literalAppend = sql``.append('raw')
  const recursiveStatement = recursiveBuilder()
  const asyncStatement = asyncBuilder()
  const boundStatement = builder.bind(null)
  const unknownStatement = factory()
  const notSql = 'raw'
  notSql.append(sql``)
  ;({}).append(sql``)
  namespaceSql``.append(sql``)
  namespaceSql.SQL``.append(sql``)
  const parts = [sql`id FROM items`]
  sql``.append(...parts)
  ;(() => undefined)()
}

export function mutableBoundary() {
  let mutableResult = sql`id FROM items`
  return mutableResult
}
