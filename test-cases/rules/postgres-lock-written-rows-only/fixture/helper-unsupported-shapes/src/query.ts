import { beginTransaction, type TransactionQuery } from "@example/db";
import { missingHelper } from "./missing";
declare const runtimeOptions: unknown;
declare const runtimeSql: string;
declare const otherWriter: (query: TransactionQuery, id: string) => Promise<void>;
async function optionalWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function* generatorWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function restWriter(query: TransactionQuery, ...ids: string[]) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [ids[0]]);
}
async function defaultWriter(query: TransactionQuery, id = "fallback") {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
async function dynamicWriter(query: TransactionQuery, id: string) {
  await query(runtimeSql, [id]);
}
async function optionWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id], runtimeOptions);
}
async function objectWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, { executor: runtimeOptions });
}
async function spreadWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [...[id]]);
}
async function mutatedWriter(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
}
mutatedWriter = otherWriter;
const arrowWriter = async (query: TransactionQuery, id: string) => {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
};
const expressionWriter = async function(query: TransactionQuery, id: string) {
  await query(`UPDATE users SET value = 1 WHERE id = $1`, [id]);
};
export async function callerOptional(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await optionalWriter?.(tx.query, id);
}
export async function callerGenerator(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await generatorWriter(tx.query, id);
}
export async function callerRest(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await restWriter(tx.query, id);
}
export async function callerDefault(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await defaultWriter(tx.query, id);
}
export async function callerDynamic(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await dynamicWriter(tx.query, id);
}
export async function callerOptions(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await optionWriter(tx.query, id);
}
export async function callerObject(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await objectWriter(tx.query, id);
}
export async function callerSpread(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await spreadWriter(tx.query, id);
}
export async function callerMutated(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await mutatedWriter(tx.query, id);
}
export async function callerArrow(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await arrowWriter(tx.query, id);
}
export async function callerExpression(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await expressionWriter(tx.query, id);
}
export async function callerMissing(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await missingHelper(tx.query, id);
}
export async function callerShadowed(id: string, optionalWriter: typeof otherWriter) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await optionalWriter(tx.query, id);
}
export async function callerSpreadArguments(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
  await optionalWriter(...[], tx.query, id);
}
