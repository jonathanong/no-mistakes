import type { SqlStatement } from "./sql";
export type SessionExecutor = (statement: string | SqlStatement) => Promise<unknown>;
export declare function beginTransaction(): Promise<SessionExecutor>;
export declare function write(
  statement: unknown,
  options: unknown,
  extra?: unknown,
): Promise<unknown>;
