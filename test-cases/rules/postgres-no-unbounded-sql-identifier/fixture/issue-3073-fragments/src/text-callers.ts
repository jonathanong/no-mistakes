import { query } from "@example/db";
import { sql } from "@example/sql";

declare function makeRaw(value: string): unknown;

function directSink(value) {
  const directCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${directCallerSlot.text}`);
}
directSink(sql.raw(process.env.CALLER_TABLE));

function aliasSink(value) {
  const aliasCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${aliasCallerSlot.text}`);
}
const raw = sql.raw(process.env.ALIAS_CALLER_TABLE);
const alias = raw;
aliasSink(alias);

function chainSink(value) {
  const chainCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${chainCallerSlot.text}`);
}
function first(value) { chainSink(value); }
function second(value) { first(value); }
second(sql.raw(process.env.CHAIN_CALLER_TABLE));

function objectSink({ value }) {
  const objectCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${objectCallerSlot.text}`);
}
objectSink({ value: sql.raw(process.env.OBJECT_CALLER_TABLE) });

function defaultSink(value) {
  const defaultCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${defaultCallerSlot.text}`);
}
function defaultForward(value = sql.raw(process.env.DEFAULT_CALLER_TABLE)) {
  defaultSink(value);
}
defaultForward();

function optionalSink(value) {
  const optionalCallerSlot = sql`${value}`;
  query(`SELECT * FROM ${optionalCallerSlot.text}`);
}
function optionalForward(value = makeRaw?.("raw")) {
  optionalSink(value);
}
optionalForward();

function ordinarySink(value) {
  const ordinaryCallerSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${ordinaryCallerSlot.text}`);
}
const ordinary = "ordinary";
ordinarySink(ordinary);
export function ordinaryForward(value) { ordinarySink(value); }

function ordinaryMemberSink({ value }) {
  const ordinaryMemberSlot = sql`records WHERE value = ${value}`;
  query(`SELECT * FROM ${ordinaryMemberSlot.text}`);
}
ordinaryMemberSink({ value: "ordinary" });

function diamondSink(value) {
  const ordinaryDiamondSlot = sql`records WHERE value = ${value} OR value = ${value}`;
  query(`SELECT * FROM ${ordinaryDiamondSlot.text}`);
}
function left(value) { diamondSink(value); }
function right(value) { diamondSink(value); }
export function diamondForward(value) { left(value); right(value); }
diamondForward("ordinary");
