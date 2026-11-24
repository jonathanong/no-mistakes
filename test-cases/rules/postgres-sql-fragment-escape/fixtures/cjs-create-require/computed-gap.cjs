const { widgetColumns } = require("./columns");
const load = require("module").createRequire(__filename);

function loadDynamic(target) {
  load(target);
  return typeof widgetColumns === "object";
}

module.exports = { loadDynamic };
