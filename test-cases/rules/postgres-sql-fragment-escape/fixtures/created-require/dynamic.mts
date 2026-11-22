import { createRequire } from "module";

const load = createRequire(import.meta.url);

export function loadModule(specifier: string) {
  return load(specifier);
}
