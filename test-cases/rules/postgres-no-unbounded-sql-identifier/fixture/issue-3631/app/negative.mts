import { read } from "@example/db";
import type {
  Annotated,
  Mutable,
  Dynamic,
  Mixed,
  Spread,
  Written,
  Escaped,
} from "./unsafe-kinds.mts";
export function annotated(k: Annotated) {
  return read(`SELECT * FROM ${k}`);
}
export function mutable(k: Mutable) {
  return read(`SELECT * FROM ${k}`);
}
export function dynamic(k: Dynamic) {
  return read(`SELECT * FROM ${k}`);
}
export function mixed(k: Mixed) {
  return read(`SELECT * FROM ${k}`);
}
export function spread(k: Spread) {
  return read(`SELECT * FROM ${k}`);
}
export function written(k: Written) {
  return read(`SELECT * FROM ${k}`);
}
export function escaped(k: Escaped) {
  return read(`SELECT * FROM ${k}`);
}

import { MUTABLE } from "./unsafe-kinds.mts";
export function directMutable(k: (typeof MUTABLE)[number]) {
  return read(`SELECT * FROM ${k}`);
}
import * as kinds from "./kinds.mts";
export function namespaceTuple(k: (typeof kinds)[number]) {
  return read(`SELECT * FROM ${k}`);
}
