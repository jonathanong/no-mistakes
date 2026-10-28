// The resolved module-scope binding cannot prove a nested binding's root.
const fs = require("node:fs");
fs.readdirSync("fixtures");
function scanNested() {
  const fs = require("node:fs");
  return fs.readdirSync("src");
}
scanNested();
