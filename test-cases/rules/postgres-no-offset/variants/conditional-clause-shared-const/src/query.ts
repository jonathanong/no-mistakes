import { read } from "@example/db";

declare const flagGlobal: boolean;
const OFF = "OFFSET 5";
const MOD = flagGlobal
  ? "SELECT id FROM accounts OFFSET 8"
  : "SELECT id FROM accounts";

export function r01(flag: boolean) {
  return read(`SELECT id FROM accounts ${flag ? OFF : ""}`); // findings: offset
}

export function r02(flag: boolean) {
  return read(`SELECT id FROM accounts ${flag ? OFF : ""}`); // findings: offset
}

export function m1() {
  return read(`${MOD}`); // findings: offset
}

export function m2() {
  return read(`${MOD}`); // findings: offset
}

export function recoveredArm(flag: boolean) {
  return read(`SELECT id
    FROM accounts
    ${flag ? "LIMIT 1" : "OFFSET 5"}`); // findings: offset
}

export function abstainedArm(flag: boolean, extra?: string) {
  return read(`SELECT id FROM accounts ${flag ? "OFFSET 5" : extra}`); // findings: unanalyzable
}

export function suppressed(flag: boolean) {
  // no-mistakes-disable-next-line postgres-no-offset
  return read(`SELECT id FROM accounts ${flag ? OFF : ""}`);
}
