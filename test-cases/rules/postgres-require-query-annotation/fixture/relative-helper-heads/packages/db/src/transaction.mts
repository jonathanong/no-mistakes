export function openTransaction() {
  return { query(_sql: unknown) {} };
}
export type TxExecutor = ReturnType<typeof openTransaction>;
