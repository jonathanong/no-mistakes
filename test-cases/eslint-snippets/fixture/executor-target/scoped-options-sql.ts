import { read } from "@example/db";
import type {
  AliasOptions,
  OptionalOptions,
  QueryOptions,
  StructuralOptions,
} from "./scoped-options-contract";

declare function unknown(value: unknown): void;

export function manualDestructured({ query: execute }: QueryOptions) {
  execute("BEGIN");
}
export function manualMember(options: QueryOptions) {
  options.query("COMMIT");
}
export function manualAlias({ execute: run }: AliasOptions) {
  run("ROLLBACK");
}
export function manualCustom(options: AliasOptions) {
  options.execute("BEGIN");
}

export function fanoutDestructured({ query: execute }: QueryOptions, items: string[]) {
  return Promise.all(items.map(() => execute("SELECT 1")));
}
export function fanoutMember(options: AliasOptions, items: string[]) {
  return Promise.all(items.map(() => options.execute("SELECT 1")));
}
export function fanoutQueryMember(options: QueryOptions, items: string[]) {
  return Promise.all(items.map(() => options.query("SELECT 1")));
}

export function adapterDestructured({ query: execute }: QueryOptions) {
  unknown(execute);
}
export function adapterMember(options: AliasOptions) {
  unknown(options.execute);
}
export function adapterQueryMember(options: QueryOptions) {
  unknown(options.query);
}
adapterDestructured({ query: read });
adapterMember({ execute: read });
adapterQueryMember({ query: read });

// These declarations have a callable shape but no required configured executor property.
export function optional(options: OptionalOptions) {
  options.execute?.("BEGIN");
}
export function structural(options: StructuralOptions) {
  options.execute("BEGIN");
}
export function computed({ ["execute"]: run }: AliasOptions) {
  run("BEGIN");
}
export function defaulted({ execute: run = read }: AliasOptions) {
  run("BEGIN");
}
// A same-named parameter in a nested function is a different binding.
export function shadowed(options: AliasOptions) {
  function inner(options: StructuralOptions) {
    options.execute("BEGIN");
    unknown(options.execute);
  }
  return inner;
}
// A runtime property selector is not the declared executor property.
export function dynamic(options: AliasOptions, key: string) {
  unknown(options[key]);
}
