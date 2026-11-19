import type {
  LooseOptions,
  QueryExecutor,
  QueryOptions,
  WideOptions,
  UnresolvedOptions,
} from "@acme/pg-core";
export type { LooseOptions, QueryExecutor, QueryOptions, UnresolvedOptions, WideOptions };
export declare function read(
  statement: string | { text: string },
  options?: QueryOptions | LooseOptions | WideOptions,
): Promise<unknown>;
export declare function write(
  statement: { text: string },
  options: QueryOptions | LooseOptions | WideOptions,
): Promise<unknown>;
