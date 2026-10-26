import { read } from "@example/db";
import SQL from "sql-template-strings";
declare function external(...values: unknown[]): unknown;

// Rejected syntax must stay Unknown rather than disappear or become SQL text.
function spreadCall() {
  return external(...["SELECT 1"]);
}
function appendArity() {
  return SQL`SELECT 1`.append(" ", " ");
}
function memberCall() {
  return { query: external }.query("SELECT 1");
}
function unboundCall() {
  return unboundSqlFactory();
}
function unsupportedBinary() {
  return 1 < 2 ? "SELECT 1" : "SELECT 2";
}
function computedObject() {
  return { ["text"]: "SELECT 1" }.text;
}
function numericObject() {
  return { 0: "SELECT 1" }["0"];
}
function prototypeObject() {
  return { __proto__: { text: "SELECT 1" } }.text;
}
function arrayHole() {
  return [, "SELECT 1"]["1"];
}
function arraySpread() {
  return [...["SELECT 1"]]["0"];
}
function numericParameter({ 0: text } = { 0: "SELECT 1" }) {
  return text;
}
function prototypeParameter({ __proto__: text } = { __proto__: "SELECT 1" }) {
  return text;
}
function nestedParameter({ value: { text } = { text: "SELECT 1" } } = {}) {
  return text;
}
function thrownBody() {
  throw "unsupported effect";
}
function optionalCall() {
  return external?.("SELECT 1");
}

read(spreadCall()); // ADVISE_SPREAD_CALL
read(appendArity()); // ADVISE_APPEND_ARITY
read(memberCall()); // ADVISE_MEMBER_CALLEE
read(unboundCall()); // ADVISE_UNBOUND_CALLEE
read(unsupportedBinary()); // ADVISE_BINARY
read(computedObject()); // ADVISE_COMPUTED_OBJECT
read(numericObject()); // ADVISE_NUMERIC_OBJECT
read(prototypeObject()); // ADVISE_PROTOTYPE_OBJECT
read(arrayHole()); // ADVISE_ARRAY_HOLE
read(arraySpread()); // ADVISE_ARRAY_SPREAD
read(numericParameter()); // ADVISE_NUMERIC_PARAMETER
read(prototypeParameter()); // ADVISE_PROTOTYPE_PARAMETER
read(nestedParameter()); // ADVISE_NESTED_PARAMETER
read(thrownBody()); // ADVISE_THROW
read(optionalCall()); // ADVISE_OPTIONAL_CALL

// Quoted own-property parameters and empty statements remain supported.
function quotedParameter({ "sql-text": text = "SELECT 1" } = {}) {
  if (text);
  else return "SELECT 1";
  return text;
}
read(quotedParameter()); // CLEAR_QUOTED_PARAMETER

function unboundQueryMember() {
  return globalThis.query("SELECT 1");
}
read(unboundQueryMember()); // ADVISE_UNBOUND_QUERY_MEMBER
