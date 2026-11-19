declare namespace pg {
  interface Pool {}
  interface PoolClient {}
}
import type { QueryExecutor } from "./executor";
export type { QueryExecutor };
type Loose = (statement: { text: string }) => Promise<unknown>;
export interface QueryOptions {
  client?: pg.PoolClient | pg.Pool;
  query?: QueryExecutor;
  readOnly?: boolean;
}
export interface LooseOptions {
  query?: Loose;
  readOnly?: boolean;
}
export interface WideOptions {
  query?: QueryExecutor;
  replica?: QueryExecutor;
}
export interface UnresolvedOptions {
  query?: Missing;
}
