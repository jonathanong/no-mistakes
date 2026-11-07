import { query } from "@example/db";
import { sql } from "@example/sql";

declare function makeRaw(value: string): unknown;
declare const flag: boolean;

function optionalSink(value) {
  const opaqueOptionalSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueOptionalSlot.text}`);
}
optionalSink(sql.raw?.(process.env.OPTIONAL_ACTUAL_TABLE));

function factorySink(value) {
  const opaqueFactorySlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueFactorySlot.text}`);
}
factorySink(makeRaw?.("raw"));

function aliasSink(value) {
  const opaqueAliasSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueAliasSlot.text}`);
}
const optional = sql.raw?.(process.env.OPTIONAL_ALIAS_TABLE);
const alias = optional;
aliasSink(alias);

function objectSink({ value }) {
  const opaqueObjectSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueObjectSlot.text}`);
}
const object = { value: sql.raw?.(process.env.OPTIONAL_OBJECT_TABLE) };
objectSink(object);

function mixedSink(value) {
  const opaqueMixedSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueMixedSlot.text}`);
}
mixedSink(flag ? "ordinary" : makeRaw?.("raw"));

function duplicateSink({ value }) {
  const opaqueDuplicateSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueDuplicateSlot.text}`);
}
duplicateSink({ value: "ordinary", value: makeRaw?.("raw") });

function literalSink(value) {
  const literalActualSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${literalActualSlot.text}`);
}
literalSink("ordinary");
literalSink(42);

function literalAliasSink(value) {
  const literalAliasActualSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${literalAliasActualSlot.text}`);
}
const literal = "ordinary";
const literalAlias = literal;
literalAliasSink(literalAlias);

function literalObjectSink({ value }) {
  const literalObjectActualSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${literalObjectActualSlot.text}`);
}
literalObjectSink({ value: "ordinary" });

function literalArraySink([value]) {
  const literalArrayActualSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${literalArrayActualSlot.text}`);
}
literalArraySink(["ordinary"]);

function literalConditionalSink(value) {
  const literalConditionalActualSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${literalConditionalActualSlot.text}`);
}
literalConditionalSink(flag ? "first" : "second");

function spreadObjectSink({ value }) {
  const opaqueSpreadObjectSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueSpreadObjectSlot.text}`);
}
spreadObjectSink({ value: "ordinary", ...{ value: makeRaw?.("raw") } });

function computedObjectSink({ value }) {
  const opaqueComputedObjectSlot = sql`${value}`;
  query(`SELECT * FROM ${opaqueComputedObjectSlot.text}`);
}
declare const key: string;
computedObjectSink({ value: "ordinary", [key]: makeRaw?.("raw") });

function prototypeSink({ value }) {
  const opaquePrototypeSlot = sql`${value}`;
  query(`SELECT * FROM ${opaquePrototypeSlot.text}`);
}
prototypeSink({ __proto__: { value: sql.raw(process.env.PROTOTYPE_TABLE) } });

function accessorSink(config) {
  config.other;
  const opaqueAccessorSlot = sql`${config.value}`;
  query(`SELECT * FROM ${opaqueAccessorSlot.text}`);
}
accessorSink({
  value: "ordinary",
  get other() { this.value = sql.raw(process.env.ACCESSOR_TABLE); return ""; },
});
