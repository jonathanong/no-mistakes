export async function beginLocalTx<R>(fn: () => Promise<R>): Promise<R> {
  return fn();
}
