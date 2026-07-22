import fs from 'node:fs';

// The exported parent must keep this `api/load` resource reachable.
export const api = {
  load() {
    fs.readFile('exported-object.json');
  },
};
api.load();

// The exported parent must keep this `Service/load` resource reachable.
export class Service {
  load() {
    fs.readFile('exported-class.json');
  }
}

// Program-scope declarations shadow the CommonJS and URL globals before their
// textual declarations, exactly as hoisting/TDZ require.
require('fs').readFile('hidden-by-program-require.json');
fs.readFile(new URL('./hidden-by-program-url.json', import.meta.url));
const require = localRequire;
const URL = LocalUrl;

function hoistedControlFlow() {
  fs.readFile('hidden-by-nested-var.json');
  if (condition) {
    var fs = localFs;
  }
}

function loopLexicalScopes() {
  for (let fs = localFs; condition; update) {
    fs.readFile('hidden-by-for.json');
  }
  for (const fs of values) {
    fs.readFile('hidden-by-for-of.json');
  }
  for (const fs in values) {
    fs.readFile('hidden-by-for-in.json');
  }
  fs.readFile('after-loops.json');
}
