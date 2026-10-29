// A wildcard body that re-exports everything can reference a production export.
declare module "*.css" {
  export * from "./api.mjs";
}
