import { query } from "@example/db";

declare function consume(value: unknown): void;

const MUTATION_ROWS = [{ key: "user", table: "users" }];
function mutationRow(key: string) {
  const selected = MUTATION_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function mutationResult() {
  const selected = mutationRow("user");
  selected.table = process.env.TABLE;
  await query(`SELECT * FROM ${selected.table}`);
}

const SECOND_ROWS = [{ key: "user", table: "users" }];
function secondRow(key: string) {
  const selected = SECOND_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function secondCaller() {
  const selected = secondRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}
export function unsafeSecondCaller() {
  const selected = secondRow("user");
  selected.table = process.env.TABLE;
}

const ALIAS_ROWS = [{ key: "user", table: "users" }];
function aliasRow(key: string) {
  const selected = ALIAS_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function aliasResult() {
  const selected = aliasRow("user");
  const alias = selected;
  alias.table = process.env.TABLE;
  await query(`SELECT * FROM ${selected.table}`);
}

const EXPORT_ROWS = [{ key: "user", table: "users" }];
export function exportedRow(key: string) {
  const selected = EXPORT_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function exportedResult() {
  const selected = exportedRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const VALUE_ROWS = [{ key: "user", table: "users" }];
function escapedValueRow(key: string) {
  const selected = VALUE_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
consume(escapedValueRow);
export async function escapedFunctionValue() {
  const selected = escapedValueRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const VALUE_OF_ROWS = [{ key: "user", table: "users" }];
function valueOfRow(key: string) {
  const selected = VALUE_OF_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function valueOfResult() {
  const selected = valueOfRow("user");
  const alias = selected.valueOf();
  alias.table = process.env.TABLE;
  await query(`SELECT * FROM ${selected.table}`);
}

const RAW_RESULT_ROWS = [{ key: "user", table: "users" }];
function rawResultRow(key: string) {
  const selected = RAW_RESULT_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
consume(rawResultRow("user"));
export async function rawResult() {
  const selected = rawResultRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const CONSTRUCTOR_ROWS = [{ key: "user", table: "users" }];
function constructorRow(key: string) {
  const selected = CONSTRUCTOR_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
const constructed = new constructorRow("user");
constructed.table = process.env.TABLE;
export async function constructorResult() {
  const selected = constructorRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const CALL_ROWS = [{ key: "user", table: "users" }];
function calledRow(key: string) {
  const selected = CALL_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
const called = calledRow.call(null, "user");
called.table = process.env.TABLE;
export async function calledResult() {
  const selected = calledRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const TAG_ROWS = [{ key: "user", table: "users" }];
function taggedRow(key: string) {
  const selected = TAG_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
taggedRow`user`;
export async function taggedResult() {
  const selected = taggedRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const REASSIGNED_ROWS = [{ key: "user", table: "users" }];
function reassignedRow(key: string) {
  const selected = REASSIGNED_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
reassignedRow = (key) => ({ key, table: process.env.TABLE });
export async function reassignedResult() {
  const selected = reassignedRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const FIND_ROWS = [{ key: "user", table: "users" }];
function rawFindRow(key: string) {
  return FIND_ROWS.find((entry) => entry.key === key);
}
export async function rawFindResult() {
  const selected = rawFindRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const UNKNOWN_ROWS = [{ key: "user", table: "users" }];
function unknownFieldRow(key: string) {
  const selected = UNKNOWN_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function unknownFieldResult() {
  const selected = unknownFieldRow("user");
  void selected.missing;
  await query(`SELECT * FROM ${selected.table}`);
}

const SHARED_ROWS = [{ key: "user", table: "users" }];
function privateSharedRow(key: string) {
  const selected = SHARED_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export function publicSharedRow(key: string) {
  const selected = SHARED_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function sharedSource() {
  const selected = privateSharedRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const DIRECT_WRITE_ROWS = [{ key: "user", table: "users" }];
function directWriteRow(key: string) {
  const selected = DIRECT_WRITE_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
directWriteRow("user").table = process.env.TABLE;
export async function directWriteResult() {
  const selected = directWriteRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const ALIAS_ONLY_ROWS = [{ key: "user", table: "users" }];
function aliasOnlyRow(key: string) {
  const selected = ALIAS_ONLY_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export async function aliasOnlyResult() {
  const selected = aliasOnlyRow("user");
  const alias = selected;
  void alias.table;
  await query(`SELECT * FROM ${selected.table}`);
}

const ALIAS_EXPORT_ROWS = [{ key: "user", table: "users" }];
function aliasedExportRow(key: string) {
  const selected = ALIAS_EXPORT_ROWS.find((entry) => entry.key === key);
  if (!selected) throw new Error("Unknown relation");
  return selected;
}
export { aliasedExportRow as publicRow };
export async function aliasedExportResult() {
  const selected = aliasedExportRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}

const METHOD_ROWS = [{ key: "user", table: "users" }];
const methods = {
  getRow(key: string) {
    const selected = METHOD_ROWS.find((entry) => entry.key === key);
    if (!selected) throw new Error("Unknown relation");
    return selected;
  },
};
export async function methodResult() {
  const selected = methods.getRow("user");
  await query(`SELECT * FROM ${selected.table}`);
}
