import { fn } from "./fn.mts";

function main(root: string) {
  fn(root);
}

main(process.argv[2] ?? process.cwd());
