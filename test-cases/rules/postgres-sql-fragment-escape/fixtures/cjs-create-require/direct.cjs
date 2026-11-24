const load = require("node:module").createRequire(__filename);

function mutate(extra) {
  const parts = load("./columns");
  parts.append(extra);
}

module.exports = { mutate };
