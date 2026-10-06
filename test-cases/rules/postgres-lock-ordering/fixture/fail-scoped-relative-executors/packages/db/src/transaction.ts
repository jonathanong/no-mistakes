export async function openTransaction() {
  return async (_sql: string, _args?: unknown[]) => {};
}

export type TxExecutor = (sql: string, args?: unknown[]) => Promise<unknown>;
