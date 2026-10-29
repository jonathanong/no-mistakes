// A wildcard body that re-exports can reference a production export.
declare module "*.css" {
  export { value } from "./api.mjs";
}
