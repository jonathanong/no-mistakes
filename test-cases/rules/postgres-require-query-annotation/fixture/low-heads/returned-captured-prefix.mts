import { read } from '@bench/db'
import sql from 'sql-template-strings'

function arbitrary() {
  const statement = sql`/* arbitrary */ SELECT 1`
  handOff(() => { statement.append(dynamicTail) })
  return statement
}
function repeated() {
  const statement = sql`/* repeated */ SELECT 1`
  for (const item of items) {
    handOff(() => { statement.append(item) })
  }
  return statement
}
function deferred() {
  const statement = sql`/* deferred */ SELECT 1`
  queueMicrotask(() => { statement.append(dynamicTail) })
  return statement
}
function appendOnly(statement: unknown) { statement.append(dynamicTail) }
function provenEffect() {
  const statement = sql`/* provenEffect */ SELECT 1`
  handOff(() => { appendOnly(statement) })
  return statement
}
function capturedReturn() {
  const statement = sql`/* capturedReturn */ SELECT 1`
  const escape = () => statement
  return statement
}
function usedAppend() {
  const statement = sql`/* usedAppend */ SELECT 1`
  handOff(() => statement.append(dynamicTail))
  return statement
}
function mutated() {
  const statement = sql`/* mutated */ SELECT 1`
  handOff(() => { statement.text = 'SELECT 1' })
  return statement
}
function unknownEffect(statement: unknown) { opaque(statement) }
function unsafeEffect() {
  const statement = sql`/* unsafeEffect */ SELECT 1`
  handOff(() => { unknownEffect(statement) })
  return statement
}
function alias() {
  const statement = sql`/* alias */ SELECT 1`
  const other = statement
  handOff(() => { statement.append(dynamicTail) })
  return statement
}
function suffixReadsBuilder() {
  const statement = sql`/* suffixReadsBuilder */ SELECT 1`
  handOff(() => { statement.append(makeTail(statement)) })
  return statement
}

read(arbitrary()) // low: true
read(repeated()) // low: true
read(deferred()) // low: true
read(provenEffect()) // low: true
read(capturedReturn()) // low: false
read(usedAppend()) // low: false
read(mutated()) // low: false
read(unsafeEffect()) // low: false
read(alias()) // low: false
read(suffixReadsBuilder()) // low: false
