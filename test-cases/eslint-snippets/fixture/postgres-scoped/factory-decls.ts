import { openTransaction } from '@example/db'

export async function viaConst() {
  const tx = await openTransaction()
  return tx('BEGIN')
}

export function viaConstNoAwait() {
  const tx = openTransaction()
  return tx('BEGIN')
}

export async function viaLet() {
  let tx = await openTransaction()
  return tx('BEGIN')
}

export async function viaUsing() {
  using tx = openTransaction()
  return tx('BEGIN')
}

export async function viaAwaitUsing() {
  await using tx = await openTransaction()
  return tx('BEGIN')
}

export async function viaMember() {
  const tx = (await openTransaction()) as unknown
  return (tx as any).query('BEGIN')
}

// `var` hoists past its block, so it is not a scoped binding.
export async function viaVar() {
  var tx = await openTransaction()
  return tx('BEGIN')
}
