import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export function load() {
  const require = (specifier: string) => specifier;
  return require("../widget.cjs");
}

require("../item.cjs");
