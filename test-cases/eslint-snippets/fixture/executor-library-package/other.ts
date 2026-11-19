import type { Statement } from "sql-template-strings";

export type QueryExecutor = (statement: Statement) => Promise<unknown>;
export interface LookalikeOptions {
  query?: QueryExecutor;
}
export declare function lookup(statement: Statement, options: LookalikeOptions): Promise<unknown>;
export declare function runWith(run: (statement: Statement) => Promise<unknown>): Promise<unknown>;
