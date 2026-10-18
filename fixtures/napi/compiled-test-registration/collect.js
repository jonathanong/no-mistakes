const { resolve } = require("node:path");

const registrations = [];
for (const entry of process.argv.slice(2)) {
  // Each Vitest entry has its own module cache, even when it requires another test file.
  for (const path of Object.keys(require.cache)) delete require.cache[path];
  const names = [];
  globalThis.test = (name) => names.push(name);
  require(resolve(entry));
  registrations.push({ entry, names });
}
process.stdout.write(JSON.stringify(registrations));
