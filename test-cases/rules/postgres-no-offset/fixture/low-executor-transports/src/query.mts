import { read, write, beginTransaction, runWithTransaction } from '@example/db';
import type { QueryExecutor, TransactionQuery, QueryOptions } from '@example/executor-types';
import type { QueryExecutor as Unconfigured, QueryOptions as OtherOptions } from '@unconfigured/executor-types';
declare const dynamicSql: string;
declare const fallback: unknown;

// Configured defaults are executor values; the function header executes no SQL.
export function defaultRead(query: QueryExecutor = read) { // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function defaultWrite(query: QueryExecutor = write) { // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function directTyped(query: QueryExecutor) { // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function dynamicStatement(query: QueryExecutor = read) { // NON_SQL
  query(dynamicSql); // ADVISE
}
// An untyped parameter remains outside the low contract at its actual call.
export function untypedDefault(query = read) { // NON_SQL
  query('SELECT 1'); // ADVISE
}
export function unknownDefault(query: QueryExecutor = fallback) { // NON_SQL
  query('SELECT 1'); // ADVISE
}
// Import spelling alone cannot make another module's annotation configured.
export function unconfiguredType(query: Unconfigured = read) { // NON_SQL
  query('SELECT 1'); // ADVISE
}

export async function callbackTyped() {
  return runWithTransaction(read, async (query: TransactionQuery) => { // NON_SQL
    await query('SELECT 1'); // CLEAR
  });
}
export async function callbackDynamic() {
  return runWithTransaction(read, async (query: TransactionQuery) => { // NON_SQL
    await query(dynamicSql); // ADVISE
  });
}
export async function factoryLiteral() {
  await using query = await beginTransaction(); // NON_SQL
  await query('SELECT 1'); // CLEAR
}
export async function factoryDynamic() {
  await using query = await beginTransaction(); // NON_SQL
  await query(dynamicSql); // ADVISE
}
export function reassignedFactory() {
  let query = beginTransaction(); // NON_SQL
  query = fallback; // NON_SQL
  query('SELECT 1'); // ADVISE
}
export function aliasedImport() {
  const query = read; // NON_SQL
  query('SELECT 1'); // ADVISE
}
export function definiteOffset(query: QueryExecutor = read) { // NON_SQL
  query('SELECT 1 OFFSET 1'); // OFFSET
}

export function typedMemberMutation(query: QueryExecutor) { // NON_SQL
  query.query = fallback; // NON_SQL
  query.query('SELECT 1'); // ADVISE
}
export function factoryMemberMutation() {
  const query = beginTransaction(); // NON_SQL
  query['query'] = fallback; // NON_SQL
  query.query('SELECT 1'); // ADVISE
}
export function computedKeyRead(query: QueryExecutor) { // NON_SQL
  const seen = {};
  seen[query] = true; // NON_SQL
  query('SELECT 1'); // CLEAR
}

export function typedAlias(query: QueryExecutor) { // NON_SQL
  const alias = query; // NON_SQL
  alias(dynamicSql); // ADVISE
}
export function typedMemberAlias(query: QueryExecutor) { // NON_SQL
  const alias = query.query; // NON_SQL
  alias(dynamicSql); // ADVISE
}
export function typedAliasQuery(query: QueryExecutor) { // NON_SQL
  const alias = query; // NON_SQL
  alias.query(dynamicSql); // ADVISE
}
export function typedCallApply(query: QueryExecutor) { // NON_SQL
  query.call(null, dynamicSql); // ADVISE
  query.apply(null, [dynamicSql]); // ADVISE
}
export function opaqueHelper(query: QueryExecutor) { // NON_SQL
  consume(query); // ADVISE
}
function consume(value: unknown) {}

export function factoryAlias() {
  const query = beginTransaction(); // NON_SQL
  const alias = query; // NON_SQL
  alias(dynamicSql); // ADVISE
}

export function unrelatedMembers(query: QueryExecutor, options: QueryOptions) { // NON_SQL
  query.release(); // NON_SQL
  const label = query.name; // NON_SQL
  const field = options.hostnameId; // NON_SQL
  options.hostnameId.toString(); // NON_SQL
  query.name.toString(); // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function memberDefault({ query = read }: QueryOptions) { // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function renamedMemberDefault({ query: run = write }: QueryOptions) { // NON_SQL
  run('SELECT 1'); // CLEAR
}
export function directMember({ query }: QueryOptions) { // NON_SQL
  query('SELECT 1'); // CLEAR
}
export function unknownMemberDefault({ query = fallback }: QueryOptions) { // NON_SQL
  query('SELECT 1'); // ADVISE
}
export function otherMember({ other = read }: QueryOptions) { // NON_SQL
  other('SELECT 1'); // ADVISE
}
export function unconfiguredMember({ query = read }: OtherOptions) { // NON_SQL
  query('SELECT 1'); // ADVISE
}
export function boundExecutor(query: QueryExecutor) { // NON_SQL
  const bound = query.bind(null); // ADVISE
  bound(dynamicSql); // existing bind escape retains uncertainty
  const member = query.query.bind(query); // ADVISE
  member(dynamicSql); // existing bind escape retains uncertainty
}
export function longerReceiver(query: QueryExecutor, key: string) { // NON_SQL
  query.db.query(dynamicSql); // ADVISE
  query[key](dynamicSql); // ADVISE
  query[0].query(dynamicSql); // ADVISE
}

// Unsupported factory-result shapes retain the factory escape as a fallback.
export function unsupportedFactoryDestructure() {
  const { query } = beginTransaction(); // ADVISE
  query(dynamicSql);
}
export function opaqueFactoryResult() {
  consume(beginTransaction()); // ADVISE
}
