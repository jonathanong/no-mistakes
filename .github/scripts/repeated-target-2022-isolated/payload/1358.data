export function read(sql: string) { return sql; }
function createRuntime() {
  async function beginTransaction() {
    const execute = async (sql: string) => read(sql);
    return execute;
  }
  const beginBoundedTransaction = () => beginTransaction();
  const executeHandlerWithCursorInBatches = async (sql: string) => read(sql);
  const createAsyncGeneratorFromCursor = async (sql: string) => read(sql);
  return { beginTransaction, beginBoundedTransaction, executeHandlerWithCursorInBatches, createAsyncGeneratorFromCursor };
}
export const { beginTransaction, beginBoundedTransaction, executeHandlerWithCursorInBatches, createAsyncGeneratorFromCursor } = createRuntime();
