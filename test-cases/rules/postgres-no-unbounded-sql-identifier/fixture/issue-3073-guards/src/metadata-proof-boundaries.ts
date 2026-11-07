export {}

declare const key: string
declare const predicate: (entry: unknown) => boolean
const NAMED_CALLBACK_ROWS = [{ key: 'user', table: 'users' }]
const namedCallback = NAMED_CALLBACK_ROWS.find(predicate)
const ASYNC_CALLBACK_ROWS = [{ key: 'user', table: 'users' }]
const asyncCallback = ASYNC_CALLBACK_ROWS.find(async entry => entry.key === key)
const DESTRUCTURED_CALLBACK_ROWS = [{ key: 'user', table: 'users' }]
const destructuredCallback = DESTRUCTURED_CALLBACK_ROWS.find(({ key: rowKey }) => rowKey === key)
const LOOSE_CALLBACK_ROWS = [{ key: 'user', table: 'users' }]
const looseCallback = LOOSE_CALLBACK_ROWS.find(entry => entry.key == key)
const OPAQUE_COMPARISON_ROWS = [{ key: 'user', table: 'users' }]
const opaqueComparison = OPAQUE_COMPARISON_ROWS.find(entry => entry.key === process.env.KEY)
const COMPUTED_COMPARISON_ROWS = [{ key: 'user', table: 'users' }]
const computedComparison = COMPUTED_COMPARISON_ROWS.find(entry => entry['key'] === key)
const REVERSED_COMPARISON_ROWS = [{ key: 'user', table: 'users' }]
const reversedComparison = REVERSED_COMPARISON_ROWS.find(entry => key !== entry.key)

const AMBIGUOUS_ROWS = [{ key: 'user', table: 'users' }]
function ambiguousGuard(key: string, duplicate: boolean) {
  if (duplicate) {
    const selected = AMBIGUOUS_ROWS.find(entry => entry.key === key)
    selected?.table
  }
  const selected = AMBIGUOUS_ROWS.find(entry => entry.key === key)
  if (!selected) throw new Error('missing')
  return selected.table
}

const MIXED_ROWS = [{ key: 'user', table: 'users' }]
function mixedReturn(key: string, flag: boolean) {
  const selected = MIXED_ROWS.find(entry => entry.key === key)
  if (!selected) throw new Error('missing')
  if (flag) return selected
  return process.env.TABLE
}
mixedReturn('user', true).table

const ASYNC_ROWS = [{ key: 'user', table: 'users' }]
async function asyncRow(key: string) {
  const selected = ASYNC_ROWS.find(entry => entry.key === key)
  if (!selected) throw new Error('missing')
  return selected
}
asyncRow('user').table

const SPREAD_ROWS = [{ key: 'user', table: 'users' }]
function spreadRow(key: string) {
  const selected = SPREAD_ROWS.find(entry => entry.key === key)
  if (!selected) throw new Error('missing')
  return selected
}
const spreadResult = spreadRow(...['user'])
spreadResult.table
