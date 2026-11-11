import { scan } from "./scan.mts";
export function main(options) {
  options.nested.root = process.cwd();
  return scan(options);
}
