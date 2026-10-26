// Saved preload: Oxlint workers inherit NODE_OPTIONS, unlike an in-process mock.
const Module = require("node:module");
const load = Module._load;
Module._load = function (id, ...args) {
  if (id === "typescript" || id === "@typescript-eslint/parser") {
    const error = new Error(`Masked optional peer: ${id}`);
    error.code = "MODULE_NOT_FOUND";
    throw error;
  }
  return load.call(this, id, ...args);
};
