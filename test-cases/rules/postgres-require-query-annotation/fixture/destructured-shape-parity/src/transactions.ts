import { beginTransaction, type TransactionQuery } from '@example/psql'

export async function runInTx<Result>(
  existing: TransactionQuery | undefined,
  work: (query: TransactionQuery) => Promise<Result>,
): Promise<Result> {
  const query = existing ?? (await beginTransaction({}))
  return work(query)
}

export function swallowFailure<Args extends [string, ...unknown[]]>(
  fn: (...args: Args) => Promise<void>,
): (...args: Args) => Promise<void> {
  return (...args) => fn(...args).catch(() => undefined)
}

async function capture(label: string, run: () => Promise<unknown>, suffix?: string) {
  console.log(label, suffix)
  await run()
}

export const captureAll = swallowFailure(capture)
