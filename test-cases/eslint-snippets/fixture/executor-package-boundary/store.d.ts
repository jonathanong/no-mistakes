import type { QueryOptions } from "@other/pg-core";
export type { QueryExecutor } from "@other/pg-core";
export declare function read(statement: { text: string }, options?: QueryOptions): Promise<unknown>;
