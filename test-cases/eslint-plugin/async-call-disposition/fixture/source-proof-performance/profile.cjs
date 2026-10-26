const { Module } = require('node:module');
const { threadId } = require('node:worker_threads');
const original = Module._load;
const proxies = new WeakMap();
const stats = { reads: 0, parses: 0, resolutions: 0, files: {} };
Module._load = function(name, ...args) {
  const value = original.call(this, name, ...args);
  if (name !== 'typescript') return value;
  if (!proxies.has(value)) {
    const sys = { ...value.sys, readFile(path, ...rest) {
      stats.reads++;
      stats.files[path] = (stats.files[path] || 0) + 1;
      return value.sys.readFile(path, ...rest);
    } };
    proxies.set(value, new Proxy(value, { get(target, key) {
      if (key === 'sys') return sys;
      if (key === 'createSourceFile') return (...input) => { stats.parses++; return target[key](...input); };
      if (key === 'resolveModuleName') return (...input) => { stats.resolutions++; return target[key](...input); };
      return Reflect.get(target, key);
    } }));
  }
  return proxies.get(value);
};
process.on('exit', () => {
  if (stats.reads || stats.parses || stats.resolutions)
    process.stderr.write('NO_MISTAKES_SOURCE_PROFILE ' + JSON.stringify({ threadId, ...stats }) + '\n');
});
