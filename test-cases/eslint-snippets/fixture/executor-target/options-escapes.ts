import { read } from "@example/db";
import type { QueryExecutor } from "@example/db";
import type { CustomOptions, Options } from "./options-escape-helpers";
import {
  importedAliasDirect, importedAliasEscape, importedDestructured, importedDirect,
  importedLeak, importedOtherField, importedReturn, importedSpread, importedStore,
  importedWrite, importedObjectAliasDirect, importedObjectAliasEscape,
  importedBindingAliasDirect, importedTypeQuery, importedDynamic, importedStaticComputed,
  importedObjectAliasCustom, importedBindingAliasCustom, importedDefaultCustom,
  importedValueAliasCustom, importedArguments, importedShorthand, importedCall, importedApply, importedOptional, importedDynamicApply, importedSpreadApply, importedDynamicMethod, importedAwait, importedYield, importedWrapped,
} from "./options-escape-helpers";

declare const store: unknown[];
declare function leak(value: unknown): void;

function localStore(options: Options) { store.push(options); }
function localReturn(options: Options) { return options; }
function localSpread(options: Options) { return { ...options }; }
function localLeak(options: Options) { leak(options); }
function localWrite(options: Options) { options.query = () => 1; }
function localAliasEscape(options: Options) {
  const query = options.query;
  store.push(query);
}
function localDestructured({ query: execute }: Options) { leak(execute); }
function localDirect(options: Options) { options.query("SELECT 1"); }
function localAliasDirect(options: Options) {
  const query = options.query;
  query("SELECT 1");
}
function localOtherField(options: Options) {
  const limit = options.limit;
  options.query(`SELECT ${limit}`);
}
function inlineEscape({ query }: { query: QueryExecutor }) { leak(query); }

localStore({ query: read, limit: 1 });
localReturn({ query: read, limit: 1 });
localSpread({ query: read, limit: 1 });
localLeak({ query: read, limit: 1 });
localWrite({ query: read, limit: 1 });
localAliasEscape({ query: read, limit: 1 });
localDestructured({ query: read, limit: 1 });
localDirect({ query: read, limit: 1 });
localAliasDirect({ query: read, limit: 1 });
localOtherField({ query: read, limit: 1 });
inlineEscape({ query: read });

importedStore({ query: read, limit: 1 });
importedReturn({ query: read, limit: 1 });
importedSpread({ query: read, limit: 1 });
importedLeak({ query: read, limit: 1 });
importedWrite({ query: read, limit: 1 });
importedAliasEscape({ query: read, limit: 1 });
importedDestructured({ query: read, limit: 1 });
importedDirect({ query: read, limit: 1 });
importedAliasDirect({ query: read, limit: 1 });
importedOtherField({ query: read, limit: 1 });
function localObjectAliasDirect(options: Options) {
  const alias = options;
  alias.query("SELECT 1");
}
function localObjectAliasEscape(options: Options) {
  const alias = options;
  store.push(alias);
}
function localBindingAliasDirect(options: Options) {
  const { query: execute } = options;
  execute("SELECT 1");
}
function localTypeQuery(options: Options) {
  type Shape = typeof options;
  options.query("SELECT 1");
  return null as Shape | null;
}
localObjectAliasDirect({ query: read, limit: 1 });
localObjectAliasEscape({ query: read, limit: 1 });
localBindingAliasDirect({ query: read, limit: 1 });
localTypeQuery({ query: read, limit: 1 });
importedObjectAliasDirect({ query: read, limit: 1 });
importedObjectAliasEscape({ query: read, limit: 1 });
importedBindingAliasDirect({ query: read, limit: 1 });
importedTypeQuery({ query: read, limit: 1 });
function localDynamic(options: Options, key: keyof Options) { leak(options[key]); }
function localStaticComputed(options: Options) { options["query"]("SELECT 1"); }
localDynamic({ query: read, limit: 1 }, "query");
localStaticComputed({ query: read, limit: 1 });
importedDynamic({ query: read, limit: 1 }, "query");
importedStaticComputed({ query: read, limit: 1 });
function localObjectAliasCustom(options: CustomOptions) {
  const alias = options;
  alias.execute("BEGIN");
}
function localBindingAliasCustom(options: CustomOptions) {
  const { execute } = options;
  execute("BEGIN");
}
function localDefaultCustom(options: CustomOptions, next = options) {
  next.execute("BEGIN");
}
function localValueAliasCustom(options: CustomOptions) {
  const execute = options.execute;
  execute("BEGIN");
}
localObjectAliasCustom({ execute: read });
localBindingAliasCustom({ execute: read });
localDefaultCustom({ execute: read });
localValueAliasCustom({ execute: read });
importedObjectAliasCustom({ execute: read });
importedBindingAliasCustom({ execute: read });
importedDefaultCustom({ execute: read });
importedValueAliasCustom({ execute: read });
function localArguments(options: CustomOptions) { leak(arguments[0]); }
localArguments({ execute: read });
importedArguments({ execute: read });
function inlineAlias({ query }: { query: QueryExecutor }) {
  const run = query;
  run("SELECT 1");
}
inlineAlias({ query: read });
function localShorthand(options: Options) { return { options }; }
localShorthand({ query: read, limit: 1 });
importedShorthand({ query: read, limit: 1 });
function inlineArguments({ query }: { query: QueryExecutor }) { leak(arguments[0]); }
function inlineReassign({ query }: { query: QueryExecutor }) { query = read; query("SELECT 1"); }
function inlineTypeQuery({ query }: { query: QueryExecutor }) {
  type Executor = typeof query;
  query("SELECT 1");
  return null as Executor | null;
}
function inlineRuntimeTypeof({ query }: { query: QueryExecutor }) {
  typeof query;
  query("SELECT 1");
}
function inlineAssertAlias({ query }: { query: QueryExecutor }) {
  const run = query as QueryExecutor;
  run("SELECT 1");
}
function inlineLetAlias({ query }: { query: QueryExecutor }) {
  let run = query;
  run("SELECT 1");
}
function inlineSpread({ query }: { query: QueryExecutor }) { query(...["SELECT 1"]); }
inlineArguments({ query: read });
inlineReassign({ query: read });
inlineTypeQuery({ query: read });
function namedRuntimeTypeof(options: Options) {
  typeof options.query;
  options.query("SELECT 1");
}
namedRuntimeTypeof({ query: read, limit: 1 });
function localCall(options: Options) { options.query.call(null, "BEGIN"); }
function localApply(options: Options) { options.query.apply(null, ["BEGIN"]); }
function localOptional(options: Options) { options.query?.("BEGIN"); }
function localDynamicApply(options: Options, args: string[]) { options.query.apply(null, args); }
function localSpreadApply(options: Options, args: string[]) { options.query.apply(null, [...args]); }
function localDynamicMethod(options: Options, method: "call") { options.query[method](null, "BEGIN"); }
localCall({ query: read, limit: 1 });
localApply({ query: read, limit: 1 });
localOptional({ query: read, limit: 1 });
localDynamicApply({ query: read, limit: 1 }, ["BEGIN"]);
localSpreadApply({ query: read, limit: 1 }, ["BEGIN"]);
localDynamicMethod({ query: read, limit: 1 }, "call");
importedCall({ query: read, limit: 1 });
importedApply({ query: read, limit: 1 });
importedOptional({ query: read, limit: 1 });
importedDynamicApply({ query: read, limit: 1 }, ["BEGIN"]);
importedSpreadApply({ query: read, limit: 1 }, ["BEGIN"]);
importedDynamicMethod({ query: read, limit: 1 }, "call");
function inlineCall({ query }: { query: QueryExecutor }) { query.call(null, "BEGIN"); }
function inlineApply({ query }: { query: QueryExecutor }) { query.apply(null, ["BEGIN"]); }
function inlineOptional({ query }: { query: QueryExecutor }) { query?.("BEGIN"); }
function inlineDynamicApply({ query }: { query: QueryExecutor }, args: string[]) {
  query.apply(null, args);
}
inlineCall({ query: read });
inlineApply({ query: read });
inlineOptional({ query: read });
inlineDynamicApply({ query: read }, ["BEGIN"]);
inlineRuntimeTypeof({ query: read });
inlineAssertAlias({ query: read });
inlineLetAlias({ query: read });
inlineSpread({ query: read });
inlineTypeQuery({ query: read });
function inlineQuotedCall({ query }: { query: QueryExecutor }) { query["call"](null, "BEGIN"); }
function inlineBind({ query }: { query: QueryExecutor }) { query.bind(null); }
function inlineExtractCall({ query }: { query: QueryExecutor }) { query.call; }
function inlineCallSpread({ query }: { query: QueryExecutor }) {
  query.call(null, ...["BEGIN"]);
}
inlineQuotedCall({ query: read });
inlineBind({ query: read });
inlineExtractCall({ query: read });
inlineCallSpread({ query: read });
function localExtractCall(options: Options) { const call = options.query.call; leak(call); }
localExtractCall({ query: read, limit: 1 });
// A callable shape does not establish the configured executor type identity.
function localStructural(options: { query: (statement: string) => unknown }) {
  options.query("SELECT 1");
}
localStructural({ query: read });
// An array binding cannot receive the declared object property by name.
function localArray([query]: QueryExecutor[]) { query("SELECT 1"); }
localArray({ query: read } as unknown as QueryExecutor[]);
// Await and yield change the callee expression; SQL discovery does not unwrap them.
async function localAwait(options: Options) { (await options.query)("BEGIN"); }
function* localYield(options: Options) { (yield options.query)("BEGIN"); }
localAwait({ query: read, limit: 1 });
localYield({ query: read, limit: 1 });
importedAwait({ query: read, limit: 1 });
importedYield({ query: read, limit: 1 });
function localWrapped(options: Options) {
  (options.query as QueryExecutor)("BEGIN");
  options.query!("COMMIT");
  (options.query)("ROLLBACK");
}
function inlineWrapped({ query }: { query: QueryExecutor }) {
  (query as QueryExecutor)("BEGIN");
}
localWrapped({ query: read, limit: 1 });
inlineWrapped({ query: read });
importedWrapped({ query: read, limit: 1 });
