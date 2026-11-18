import { read } from "@example/db";
const OPEN = ["painted", "plain"];
const WIDE: readonly string[] = ["painted", "plain"] as const;
let MUTABLE = ["painted", "plain"] as const;
const MIXED = ["painted", 1] as const;
declare const external: string;
const DYNAMIC = ["painted", external] as const;
const SPREAD = [...OPEN] as const;
const TUPLE = ["painted", "plain"] as const;
type Variety = (typeof TUPLE)[number];
export function open(kind: string) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function widened(kind: (typeof OPEN)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function annotated(kind: (typeof WIDE)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function mutable(kind: (typeof MUTABLE)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function mixed(kind: (typeof MIXED)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function dynamic(kind: (typeof DYNAMIC)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function spread(kind: (typeof SPREAD)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function shadowed(TUPLE: string[], kind: (typeof TUPLE)[number]) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function optional(kind?: Variety) {
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function reassigned(kind: Variety, replacement: string) {
  kind = replacement;
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function ineffectiveGuard(kind: string) {
  if (!TUPLE.some(item => item === kind)) new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function asyncGuard(kind: string) {
  if (!TUPLE.some(async item => item === kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function looseGuard(kind: string) {
  if (!TUPLE.some(item => item == kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function unrelatedGuard(kind: string, other: string) {
  if (!TUPLE.some(item => item === other)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function overwrittenGuard(kind: string) {
  if (!TUPLE.some(item => item === kind)) throw new Error("Unknown variety");
  kind = external;
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function blockPredicate(kind: string) {
  if (!TUPLE.some(item => { return item === kind; })) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function compoundPredicate(kind: string) {
  if (!TUPLE.some(item => item === kind || kind.length > 0)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function defaultPredicate(kind: string) {
  if (!TUPLE.some((item = kind) => item === kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function extraPredicateParameter(kind: string) {
  if (!TUPLE.some((item, index) => item === kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function tautologyPredicate(kind: string) {
  if (!TUPLE.some(item => item === item)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
export function destructuredPredicate(kind: string) {
  if (!TUPLE.some(({ value }) => value === kind)) throw new Error("Unknown variety");
  return read(`SELECT id FROM ${kind} LIMIT 10`);
}
