// A wildcard body that imports can reference a production export.
declare module "*.css" {
  import { value } from "./api.mjs";
  export default value;
}
