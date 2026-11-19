import type { SqlStatement } from "./sql";
export type SessionExecutor = <T = unknown>(statement: string | SqlStatement) => Promise<T>;
export declare const read: SessionExecutor;
export declare const write: SessionExecutor;
