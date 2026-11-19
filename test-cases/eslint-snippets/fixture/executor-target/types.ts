export type QueryExecutor = (statement: string) => unknown;
export declare const read: QueryExecutor;
export declare const write: QueryExecutor;

export interface PoolClient {
  query(statement: string): unknown;
}

export interface QueryWithClient {
  (statement: string): unknown;
  client: PoolClient;
}

export interface SessionHandle {
  readonly id: string;
}
