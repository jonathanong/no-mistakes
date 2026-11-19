import type { SqlStatement } from "./sql";

export interface ExecutorOptions {
  query?: SessionExecutor;
  readOnly?: boolean;
}
export type SessionExecutor = (
  statement: string | SqlStatement,
  options?: ExecutorOptions,
) => Promise<unknown>;
export declare const read: SessionExecutor;
export declare const write: SessionExecutor;
