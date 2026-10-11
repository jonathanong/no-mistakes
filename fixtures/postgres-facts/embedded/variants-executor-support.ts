export type TxExecutor = (statement: unknown) => Promise<void>;

export function openTransaction(): TxExecutor {
  return async (_statement: unknown) => {};
}
