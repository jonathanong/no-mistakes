import type { SqlStatement } from "./sql";
export type SessionExecutor = <T = unknown>(
  statement: string | SqlStatement,
) => Promise<{ rows: T[] }>;
export declare function beginTransaction(): Promise<SessionExecutor>;
export declare function openSession(): Promise<SessionExecutor>;
declare const defaultOpener: () => Promise<SessionExecutor>;
export default defaultOpener;
