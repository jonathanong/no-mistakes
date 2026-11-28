export type QueryFn = {
  <T = unknown>(sql: string): Promise<T[]>;
  <T = unknown>(strings: TemplateStringsArray, ...values: unknown[]): Promise<T[]>;
};

export async function withTx<R>(fn: (query: QueryFn) => Promise<R>): Promise<R> {
  return fn((async () => []) as QueryFn);
}
