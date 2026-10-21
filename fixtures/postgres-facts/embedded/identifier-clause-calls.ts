import sql from "sql-template-strings";
import { read } from "@example/db";

function leaf(kind: string) {
  switch (kind) {
    case "x":
      return "orders";
    case "y":
      return "accounts";
    default:
      throw new Error("unknown table");
  }
}

function via(kind: string) {
  return leaf(kind);
}

export function nestedHole(kind: string, id: string) {
  return read(`SELECT id FROM ${via(kind)} WHERE id = $1`, [id]);
}

function dup(kind: string) {
  switch (kind) {
    case "a":
      return "orders";
    case "b":
      return "orders";
    default:
      throw new Error("unknown table");
  }
}

export function dupHole(kind: string, id: string) {
  return read(`SELECT id FROM ${dup(kind)} WHERE account_id = $1`, [id]);
}

function innerUnknown(kind: string) {
  return missing(kind);
}

function outerWrap(kind: string) {
  return innerUnknown(kind);
}

export function failedNested(kind: string, id: string) {
  return read(`SELECT id FROM ${outerWrap(kind)} WHERE account_id = $1`, [id]);
}

// d0 is run directly. The ninth nested call, d8 to d9, is past the depth cap.
function d9() {
  return "orders";
}
function d8() {
  return d9();
}
function d7() {
  return d8();
}
function d6() {
  return d7();
}
function d5() {
  return d6();
}
function d4() {
  return d5();
}
function d3() {
  return d4();
}
function d2() {
  return d3();
}
function d1() {
  return d2();
}
function d0() {
  return d1();
}

export function depthHole(id: string) {
  return read(`SELECT id FROM ${d0()} WHERE account_id = $1`, [id]);
}

function nulName(kind: string) {
  switch (kind) {
    case "x":
      return "ord\0ers";
    default:
      throw new Error("unknown table");
  }
}

export function nulHole(kind: string, id: string) {
  return read(`SELECT id FROM ${nulName(kind)} WHERE account_id = $1`, [id]);
}

function stmt(kind: string) {
  switch (kind) {
    case "x":
      return sql`SELECT id FROM orders WHERE account_id = $1`;
    default:
      throw new Error("unknown");
  }
}

export function sqlHole(kind: string) {
  return read(`SELECT ${stmt(kind)}`, [1]);
}
