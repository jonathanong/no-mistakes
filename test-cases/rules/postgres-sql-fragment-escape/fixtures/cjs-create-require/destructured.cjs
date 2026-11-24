const { createRequire } = require("module");

const load = createRequire(__filename);

function mutate(extra) {
  const parts = load("./columns");
  parts.append(extra);
}

module.exports = { mutate };
