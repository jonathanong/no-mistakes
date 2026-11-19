import { beginTransaction, type TransactionQuery } from '@example/psql'

export async function runInTx<Result>(
  existing: TransactionQuery | undefined,
  work: (query: TransactionQuery) => Promise<Result>,
): Promise<Result> {
  const query = existing ?? (await beginTransaction({}))
  return work(query)
}
