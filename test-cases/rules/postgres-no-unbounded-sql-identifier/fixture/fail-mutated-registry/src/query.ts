import { query } from "@example/db";

const CLEAN = new Map([
  ["user", "users"],
  ["post", "posts"],
]);

const M = new Map([["a", "users"]]);
M.set("b", process.env.T);

const M3 = new Map([["a", "users"]]);
function mut(v: string) {
  M3.set("b", v);
}

const M4 = new Map([["a", "users"]]);

const SET = new Set(["users"]);
SET.add(process.env.T);

const SET2 = new Set(["users"]);
function addLater(t: string) {
  SET2.add(t);
}

const ARR = ["users"];
ARR.push(process.env.T);

const ARR2 = ["users"];
ARR2[0] = process.env.T;

const M5 = new Map([["a", "users"]]);
Object.assign(M5, { b: process.env.T });

export function clean(kind: string) {
  return query(`SELECT id FROM ${CLEAN.get(kind)}`);
}

export function mapSet(kind: string) {
  return query(`SELECT id FROM ${M.get(kind)}`);
}

export function mapSetInHelper(kind: string) {
  return query(`SELECT id FROM ${M3.get(kind)}`);
}

export function mapSetBeforeGet(kind: string) {
  M4.set(kind, process.env.T);
  return query(`SELECT id FROM ${M4.get(kind)}`);
}

export function setAdd(table: string) {
  if (!SET.has(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}

export function setAddInHelper(table: string) {
  if (!SET2.has(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}

export function arrayPush(table: string) {
  if (!ARR.includes(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}

export function elementWrite(table: string) {
  if (!ARR2.includes(table)) throw new Error("table");
  return query(`SELECT id FROM ${table}`);
}

export function objectAssign(kind: string) {
  return query(`SELECT id FROM ${M5.get(kind)}`);
}

void mut;
void addLater;

const ALIAS_MAP = new Map([["a", "users"]]);
const alias = ALIAS_MAP;
alias.set("b", process.env.T);

const PASSED = new Map([["a", "users"]]);
function fill(m: Map<string, string>) {
  m.set("z", process.env.T);
}
fill(PASSED);

const RETURNED = new Map([["a", "users"]]);
function leak() {
  return RETURNED;
}

const EXPORTED = new Map([["a", "users"]]);
export { EXPORTED };

export function aliased(kind: string) {
  return query(`SELECT id FROM ${ALIAS_MAP.get(kind)}`);
}

export function passed(kind: string) {
  return query(`SELECT id FROM ${PASSED.get(kind)}`);
}

export function returned(kind: string) {
  return query(`SELECT id FROM ${RETURNED.get(kind)}`);
}

export function exported(kind: string) {
  return query(`SELECT id FROM ${EXPORTED.get(kind)}`);
}

void leak;

const WRAPPER = new Map([["a", "users"]]);
const holder = { WRAPPER };
holder.WRAPPER.set("b", process.env.T);

const ARRAY_ALIAS = new Map([["a", "users"]]);
const [zz] = [ARRAY_ALIAS];
zz.set("b", process.env.T);

export const EXPORTED_CONST = new Map([["a", "users"]]);

const ARROW = new Map([["a", "users"]]);
export const getArrow = () => ARROW;

const NESTED_ARG = new Map([["a", "users"]]);
function sink(value: object) {
  void value;
}
sink({ m: NESTED_ARG });

const DEFAULT_REG = new Map([["a", "users"]]);
export default DEFAULT_REG;

export function wrapped(kind: string) {
  return query(`SELECT id FROM ${WRAPPER.get(kind)}`);
}

export function arrayAlias(kind: string) {
  return query(`SELECT id FROM ${ARRAY_ALIAS.get(kind)}`);
}

export function exportedConst(kind: string) {
  return query(`SELECT id FROM ${EXPORTED_CONST.get(kind)}`);
}

export function arrowReturned(kind: string) {
  return query(`SELECT id FROM ${ARROW.get(kind)}`);
}

export function nestedArgument(kind: string) {
  return query(`SELECT id FROM ${NESTED_ARG.get(kind)}`);
}

export function defaultExport(kind: string) {
  return query(`SELECT id FROM ${DEFAULT_REG.get(kind)}`);
}
