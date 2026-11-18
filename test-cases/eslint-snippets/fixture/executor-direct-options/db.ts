export type SessionExecutor = (statement: unknown, ...args: unknown[]) => Promise<unknown>;
export declare const read: SessionExecutor;
export declare const write: SessionExecutor;
