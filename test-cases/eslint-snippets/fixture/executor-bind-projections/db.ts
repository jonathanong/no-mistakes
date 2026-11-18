import type { SqlStatement } from "./sql";
export type SessionExecutor = (statement: string | SqlStatement) => Promise<unknown>;
export declare const read: SessionExecutor;
export declare const write: SessionExecutor;
