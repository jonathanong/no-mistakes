// A computed require can load an enumerator even when its uses are not calls.
const moduleName = "node:fs";
const loaded = require(moduleName);
export { loaded };
