const { Module } = require('node:module');
const load = Module._load;
// Model a consumer installation lacking the two optional source-proof peers.
Module._load = function (name, parent, ...args) {
  if (parent?.filename.endsWith('/async-source-store.js') &&
      ['typescript', '@typescript-eslint/parser'].includes(name)) {
    const error = new Error(`Missing optional peer: ${name}`);
    error.code = 'MODULE_NOT_FOUND';
    throw error;
  }
  return load.call(this, name, parent, ...args);
};
