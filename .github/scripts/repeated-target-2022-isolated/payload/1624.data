import { openTransaction } from '@example/db'

export async function viaConst() {
  const tx = await openTransaction()
  return tx('SELECT id FROM const_awaited')
}

export function viaConstNoAwait() {
  const tx = openTransaction()
  return tx('SELECT id FROM const_plain')
}

export async function viaLet() {
  let tx = await openTransaction()
  return tx('SELECT id FROM let_awaited')
}

export async function viaUsing() {
  using tx = openTransaction()
  return tx('SELECT id FROM using_plain')
}

export async function viaAwaitUsing() {
  await using tx = await openTransaction()
  return tx('SELECT id FROM await_using_awaited')
}

export async function viaAsWrapper() {
  const tx = (await openTransaction()) as unknown
  return (tx as any)('SELECT id FROM wrapped')
}
