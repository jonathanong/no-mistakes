// A wildcard body with import-equals can reference a production export.
declare module "*.css" {
  import api = require("./api.cjs");
  export default api;
}
