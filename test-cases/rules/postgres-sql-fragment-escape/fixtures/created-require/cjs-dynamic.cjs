const load = require("module").createRequire(__filename);

function loadDynamic(target) {
  return load(target);
}

module.exports = { loadDynamic };
