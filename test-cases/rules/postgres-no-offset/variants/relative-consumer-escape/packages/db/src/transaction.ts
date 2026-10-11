export type TxExecutor = (statement: unknown, args?: unknown) => void;

export function openTransaction(): TxExecutor {
  return (_statement) => {};
}
