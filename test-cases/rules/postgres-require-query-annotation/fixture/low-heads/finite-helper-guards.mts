import { read } from '@bench/db'
import sql from 'sql-template-strings'

function resetGuard(flag: boolean) {
  const q = sql`/* resetGuard */ SELECT 1`
  if (q.reset()) throw Error('reset')
  if (flag) return q
  return q
}
function textGuard(flag: boolean) {
  const q = sql`/* textGuard */ SELECT 1`
  if ((q.text = 'SELECT 0')) throw Error('changed')
  if (flag) return q
  return q
}
function branchWrite(flag: boolean) {
  const q = sql`/* branchWrite */ SELECT 1`
  if (flag) { q.text = 'SELECT 0' }
  if (flag) return q
  return q
}
function escapeGuard(flag: boolean) {
  const q = sql`/* escapeGuard */ SELECT 1`
  if (flag) unknown(q)
  if (flag) return q
  return q
}
function mixedGuard(flag: boolean) {
  const q = sql`/* mixedGuard */ SELECT 1`
  if (flag) { q.setName('changed'); q.text = 'SELECT 0' }
  if (flag) return q
  return q
}
function throwMutation(flag: boolean) {
  const q = sql`/* throwMutation */ SELECT 1`
  if (flag) throw q.reset()
  if (flag) return q
  return q
}
function afterEarlyReturn(flag: boolean) {
  const q = sql`/* afterEarlyReturn */ SELECT 1`
  if (flag) return q
  q.text = 'SELECT 0'
  return q
}
function returnedMutation(flag: boolean) {
  const q = sql`/* returnedMutation */ SELECT 1`
  if (flag) return q.reset()
  return q
}
function calledClosure(flag: boolean) {
  const q = sql`/* calledClosure */ SELECT 1`
  const mutate = () => { q.text = 'SELECT 0' }
  if (flag) mutate()
  if (flag) return q
  return q
}
function assignedGuard(flag: boolean) {
  const q = sql`/* assignedGuard */ SELECT 1`
  if (flag) Object.assign(q, { text: 'SELECT 0' })
  if (flag) return q
  return q
}
function computedGuard(flag: boolean) {
  const q = sql`/* computedGuard */ SELECT 1`
  if (flag) q['text'] = 'SELECT 0'
  if (flag) return q
  return q
}
function stringsGuard(flag: boolean) {
  const q = sql`/* stringsGuard */ SELECT 1`
  if (flag) q.strings[0] = 'SELECT 0'
  if (flag) return q
  return q
}
function aliasGuard(flag: boolean) {
  const q = sql`/* aliasGuard */ SELECT 1`
  const alias = q
  if (flag) alias.text = 'SELECT 0'
  if (flag) return q
  return q
}
function appendResultGuard(flag: boolean) {
  const q = sql`/* appendResultGuard */ SELECT 1`
  if (flag) q.append(tail).text = 'SELECT 0'
  if (flag) return q
  return q
}
function nestedReturn(flag: boolean) {
  const q = sql`/* nestedReturn */ SELECT 1`
  if (flag) { if (unknown()) return q }
  return q
}
function safeAppendGuard(flag: boolean) {
  const q = sql`/* safeAppendGuard */ SELECT 1`
  if (flag) q.append(tail)
  if (flag) return q
  return q
}
function literalGuard(flag: boolean) {
  if (unknown()) return sql`/* literalGuard */ SELECT 1`
  if (flag) return sql`/* literalGuard */ SELECT 2`
  return sql`/* literalGuard */ SELECT 3`
}
const moduleQuery = sql`/* reassignedByGuard */ SELECT 1`
function reassignModule() { moduleQuery.text = 'SELECT 0' }
function moduleGuard(flag: boolean) {
  if (flag) reassignModule()
  if (flag) return moduleQuery
  return moduleQuery
}

read(resetGuard(true)) // low: false
read(textGuard(true)) // low: false
read(branchWrite(true)) // low: false
read(escapeGuard(true)) // low: false
read(mixedGuard(true)) // low: false
read(throwMutation(true)) // low: false
read(afterEarlyReturn(true)) // low: false
read(returnedMutation(true)) // low: false
read(calledClosure(true)) // low: false
read(assignedGuard(true)) // low: false
read(computedGuard(true)) // low: false
read(stringsGuard(true)) // low: false
read(aliasGuard(true)) // low: false
read(appendResultGuard(true)) // low: false
read(nestedReturn(true)) // low: false
read(safeAppendGuard(true)) // low: true
read(literalGuard(true)) // low: true
read(moduleGuard(true)) // low: false
