function use(ns: unknown) {
  return ns;
}

const parts = require("./requireValueControls");

export function load() {
  return use(parts);
}
