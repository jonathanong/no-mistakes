import { read } from "@example/db";
import SQL from "sql-template-strings";
declare const flag: boolean;

// Falsy primitive branches must not manufacture SQL from dead alternatives.
function nullTruth() {
  return null ? "invalid SQL" : "SELECT 1";
}
function undefinedTruth() {
  return undefined ? "invalid SQL" : "SELECT 1";
}
function emptyTextTruth() {
  return "" ? "invalid SQL" : "SELECT 1";
}
function nonemptyTextTruth() {
  return "present" ? "SELECT 1" : "invalid SQL";
}
read(nullTruth()); // CLEAR_NULL_TRUTH
read(undefinedTruth()); // CLEAR_UNDEFINED_TRUTH
read(emptyTextTruth()); // CLEAR_EMPTY_TEXT
read(nonemptyTextTruth()); // CLEAR_NONEMPTY_TEXT

// Unsupported coercion/member/append operands remain Unknown at the real sink.
function badConcat() {
  return "SELECT " + null;
}
function nullMember() {
  return null.text;
}
function arrayMember() {
  return ["SELECT 1"]["0"];
}
function appendTextReceiver() {
  return "SELECT 1".append(SQL` WHERE true`);
}
function appendNullSuffix() {
  const result = SQL`SELECT 1`;
  result.append(null);
  return result;
}
function dynamicPlainTemplate(value: unknown) {
  return `SELECT ${value}`;
}
read(badConcat()); // ADVISE_CONCAT_COERCION
read(nullMember()); // ADVISE_NULL_MEMBER
read(arrayMember()); // ADVISE_ARRAY_MEMBER
read(appendTextReceiver()); // ADVISE_APPEND_RECEIVER
read(appendNullSuffix()); // ADVISE_APPEND_SUFFIX
read(dynamicPlainTemplate(flag)); // ADVISE_PLAIN_TEMPLATE

// Constrained scalar alternatives cross a real helper boundary and its member read.
function inspect(value: unknown) {
  return value.field ? "SELECT 1" : "SELECT 2";
}
function constrained(value: unknown) {
  return inspect(value || false);
}
read(constrained(flag)); // ADVISE_CONSTRAINED_MEMBER: the boolean alternative cannot expose a property.

function constrainedScalar(value: unknown) {
  return inspect(value || value);
}
read(constrainedScalar(flag)); // CLEAR_CONSTRAINED_SCALAR

function looseCoercion() {
  return "1" == true ? "SELECT 1" : "SELECT 2";
}
function returnedVoid() {
  return;
}
read(looseCoercion()); // CLEAR_LOOSE_COMPARISON
read(returnedVoid()); // ADVISE_VOID_SQL_ARGUMENT

function multilineSuffix(value: unknown) {
  return SQL`
    WHERE ${value} = ${value}
    AND true
  `;
}
function composedBinds(value: unknown) {
  return SQL`SELECT ${value} AS value`.append(multilineSuffix(value));
}
read(composedBinds(flag)); // CLEAR_MULTILINE_RENUMBER

async function awaitedText() {
  return await "SELECT 1";
}
async function awaitedBoolean() {
  return (await true) ? "SELECT 1" : "invalid SQL";
}
export async function primitiveAwait() {
  read(await awaitedText()); // CLEAR_AWAIT_TEXT
  read(await awaitedBoolean()); // CLEAR_AWAIT_BOOL
}

function identity(value: unknown) {
  return value;
}
export async function promiseArgument() {
  read(await identity(awaitedText())); // CLEAR_PROMISE_ARGUMENT
}
