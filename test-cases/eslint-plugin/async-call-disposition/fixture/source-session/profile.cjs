const { Module } = require('node:module');
const original = Module._load;
const stats = { reads: {}, parses: 0, resolutions: 0 };
Module._load = function(name, ...args) {
  const absent = process.env.SOURCE_SESSION_PARSER_ABSENT === 'true';
  if (absent && name === '@typescript-eslint/parser') {
    const error = new Error("Cannot find module '@typescript-eslint/parser'");
    error.code = 'MODULE_NOT_FOUND';
    throw error;
  }
  const value = original.call(this, name, ...args);
  if (name === 'typescript') return {
    ...value,
    sys: { ...value.sys, readFile(path, ...rest) {
      stats.reads[path] = (stats.reads[path] || 0) + 1;
      return value.sys.readFile(path, ...rest);
    } },
    createSourceFile(...input) {
      if (absent) stats.parses++;
      return value.createSourceFile(...input);
    },
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
