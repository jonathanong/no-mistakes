import type { QueryExecutor } from "@example/db";

export interface QueryOptions {
  query: QueryExecutor;
}

export type AliasOptions = { execute: QueryExecutor };
export type OptionalOptions = { execute?: QueryExecutor };
export type StructuralOptions = { execute: (statement: string) => unknown };
