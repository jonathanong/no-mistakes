const box = require("@example/box");
import eq = require("@example/box");

box.value = 1;
eq.value = 2;

export function local() {
  const inner = require("@example/box");
  inner.value = 3;
}

export const tables = {};
tables.row = 1;
