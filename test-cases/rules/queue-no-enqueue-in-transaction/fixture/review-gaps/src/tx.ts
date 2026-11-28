export type QueryFn = {
  <T = unknown>(sql: string): Promise<T[]>
  items: { id: string }[]
}

export async function withTx<R>(fn: (query: QueryFn) => Promise<R>): Promise<R> {
  return fn((async () => []) as QueryFn)
}
