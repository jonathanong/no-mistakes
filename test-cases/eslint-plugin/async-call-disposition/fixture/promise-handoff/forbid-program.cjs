const Module = require('node:module');
const load = Module._load;
const constructors = new Set(['createProgram', 'createIncrementalProgram', 'createWatchProgram']);
// Real Oxlint must use the source session, never a private TypeScript program.
Module._load = function(request, parent, isMain) {
  const value = load.call(this, request, parent, isMain);
  if (request !== 'typescript') return value;
  return new Proxy(value, {
    get(target, key) {
      if (constructors.has(key)) return () => { throw new Error('Private TypeScript programs are forbidden'); };
      return Reflect.get(target, key);
    },
  });
};
