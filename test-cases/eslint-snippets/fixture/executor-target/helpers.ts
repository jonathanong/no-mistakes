import type { QueryExecutor } from "@example/db";
export function run(exec?: QueryExecutor) {}
export function object({ query }: { query: QueryExecutor }) {}
export function untyped(exec: any) {}
export function rest(...exec: QueryExecutor[]) {}
export function overloaded(exec: QueryExecutor): void;
export function overloaded(exec: string): void;
export function overloaded(exec: unknown) {}
export function structural(exec: (statement: string) => unknown) {}
export function plainObject(options: { query: QueryExecutor }) {}
export function unknownObject({ query }: any) {}
export function wrongObject({ other }: { query: QueryExecutor; other: unknown }) {}
export function singleOverload(exec: QueryExecutor): void;
export function singleOverload(exec: unknown) {}
import type { QueryExecutor as OtherExecutor } from "./types";
export function unrelated(exec: OtherExecutor) {}
export function renamed({ query: execute }: { query: QueryExecutor }) {}
export function nested({ query: { exec } }: { query: QueryExecutor }) {}
export function array([query]: { query: QueryExecutor }) {}
export function computed({ ["query"]: query }: { query: QueryExecutor }) {}
