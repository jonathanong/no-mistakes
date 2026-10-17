const { fileURLToPath: fromUrl } = require('node:url');
const nodePath = require('node:path');
const proc = require('node:process');
if (nodePath.resolve(proc.argv[1]) === __filename) main();
function shadowRequire(require) {
  const proc = require('node:process');
  if (proc.argv[1] === __filename) main();
}
const { proc: fakeProcess } = require('node:process');
if (fakeProcess.argv[1] === __filename) main();
