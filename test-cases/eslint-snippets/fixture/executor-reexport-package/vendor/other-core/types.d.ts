export type QueryExecutor = (statement: { text: string }) => Promise<unknown>;
export interface QueryOptions {
  query?: QueryExecutor;
}
