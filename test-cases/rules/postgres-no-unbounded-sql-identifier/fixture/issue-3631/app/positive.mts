import { read } from "@example/db";
import type { Kind, RenamedKind } from "./kinds.mts";
import { KINDS, RENAMED } from "./kinds.mts";
import { BARREL_KINDS } from "./barrel.mts";
import type { BarrelKind } from "./barrel.mts";
export function importedAlias(k: Kind) {
  return read(`SELECT * FROM ${k}`);
}
export function importedTuple(k: (typeof KINDS)[number]) {
  return read(`SELECT * FROM ${k}`);
}
export function renamedAlias(k: RenamedKind) {
  return read(`SELECT * FROM ${k}`);
}
export function renamedTuple(k: (typeof RENAMED)[number]) {
  return read(`SELECT * FROM ${k}`);
}
export function barrelAlias(k: BarrelKind) {
  return read(`SELECT * FROM ${k}`);
}
export function barrelTuple(k: (typeof BARREL_KINDS)[number]) {
  return read(`SELECT * FROM ${k}`);
}
export const LOCAL = ["x", "y"] as const;
export type Local = (typeof LOCAL)[number];
export function exportedLocal(k: Local) {
  return read(`SELECT * FROM ${k}`);
}
const CLAUSE = ["left", "right"] as const;
export { CLAUSE };
export function exportClause(k: (typeof CLAUSE)[number]) {
  return read(`SELECT * FROM ${k}`);
}
import type { KINDS as TYPES_ONLY } from "./kinds.mts";
import type { ForwardedKind } from "./forwarded.mts";
export function typeOnlyTuple(k: (typeof TYPES_ONLY)[number]) {
  return read(`SELECT * FROM ${k}`);
}
export function forwardedAlias(k: ForwardedKind) {
  return read(`SELECT * FROM ${k}`);
}
