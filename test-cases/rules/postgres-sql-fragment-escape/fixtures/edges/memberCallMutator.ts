export function extend(loader: {
  require(spec: string): unknown;
  import(spec: string): unknown;
}) {
  loader.require("./widgetColumns");
  return loader.import("./widgetColumns");
}
