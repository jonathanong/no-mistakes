const { Module } = require('node:module');
const original = Module._load;
const stats = { reads: {}, parses: 0, resolutions: 0 };
Module._load = function(name, ...args) {
  const value = original.call(this, name, ...args);
  if (name === 'typescript') return {
    ...value,
    sys: { ...value.sys, readFile(path, ...rest) {
      stats.reads[path] = (stats.reads[path] || 0) + 1;
      return value.sys.readFile(path, ...rest);
    } },
    resolveModuleName(...input) {
      stats.resolutions++;
      return value.resolveModuleName(...input);
    },
  };
  if (name === '@typescript-eslint/parser') return {
    ...value,
    parseForESLint(...input) {
      stats.parses++;
      return value.parseForESLint(...input);
    },
  };
  return value;
};
process.on('exit', () => {
  if (stats.parses) process.stderr.write('SOURCE_SESSION_COUNTS ' + JSON.stringify(stats) + '\n');
});
