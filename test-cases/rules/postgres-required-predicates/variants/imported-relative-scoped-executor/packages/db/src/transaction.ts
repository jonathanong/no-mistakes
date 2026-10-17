export async function openTransaction() {
  return { query: async (_statement: unknown) => {} };
}

export type TxExecutor = { query(statement: unknown): Promise<unknown> };
