import type { QueryExecutor } from "@example/db";

export interface Options {
  query: QueryExecutor;
  limit: number;
}
export type CustomOptions = { execute: QueryExecutor };

declare const store: unknown[];
declare function leak(value: unknown): void;

export function importedStore(options: Options) { store.push(options); }
export function importedReturn(options: Options) { return options; }
export function importedSpread(options: Options) { return { ...options }; }
export function importedLeak(options: Options) { leak(options); }
export function importedWrite(options: Options) { options.query = () => 1; }
export function importedAliasEscape(options: Options) {
  const query = options.query;
  store.push(query);
}
export function importedDestructured({ query: execute }: Options) { leak(execute); }
export function importedDirect(options: Options) { options.query("SELECT 1"); }
export function importedAliasDirect(options: Options) {
  const query = options.query;
  query("SELECT 1");
}
export function importedOtherField(options: Options) {
  const limit = options.limit;
  options.query(`SELECT ${limit}`);
}
export function importedObjectAliasDirect(options: Options) {
  const alias = options;
  alias.query("SELECT 1");
}
export function importedObjectAliasEscape(options: Options) {
  const alias = options;
  store.push(alias);
}
export function importedBindingAliasDirect(options: Options) {
  const { query: execute } = options;
  execute("SELECT 1");
}
export function importedTypeQuery(options: Options) {
  type Shape = typeof options;
  options.query("SELECT 1");
  return null as Shape | null;
}
export function importedDynamic(options: Options, key: keyof Options) { leak(options[key]); }
export function importedStaticComputed(options: Options) { options["query"]("SELECT 1"); }
// Whole-object and binding aliases are not connected to a custom member by SQL discovery.
export function importedObjectAliasCustom(options: CustomOptions) {
  const alias = options;
  alias.execute("BEGIN");
}
export function importedBindingAliasCustom(options: CustomOptions) {
  const { execute } = options;
  execute("BEGIN");
}
export function importedDefaultCustom(options: CustomOptions, next = options) {
  next.execute("BEGIN");
}
export function importedValueAliasCustom(options: CustomOptions) {
  const execute = options.execute;
  execute("BEGIN");
}
// `arguments` exposes a parameter without naming its binding in the body.
export function importedArguments(options: CustomOptions) { leak(arguments[0]); }
export function importedShorthand(options: Options) { return { options }; }
export function importedCall(options: Options) { options.query.call(null, "BEGIN"); }
export function importedApply(options: Options) { options.query.apply(null, ["BEGIN"]); }
export function importedOptional(options: Options) { options.query?.("BEGIN"); }
export function importedDynamicApply(options: Options, args: string[]) {
  options.query.apply(null, args);
}
export function importedSpreadApply(options: Options, args: string[]) {
  options.query.apply(null, [...args]);
}
export function importedDynamicMethod(options: Options, method: "call") {
  options.query[method](null, "BEGIN");
}
export async function importedAwait(options: Options) { (await options.query)("BEGIN"); }
export function* importedYield(options: Options) { (yield options.query)("BEGIN"); }
export function importedWrapped(options: Options) {
  (options.query as QueryExecutor)("BEGIN");
  options.query!("COMMIT");
  (options.query)("ROLLBACK");
}
