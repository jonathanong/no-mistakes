import { listFiles } from "./list.mts";
// The property key matches the import local, so the module is consumed without a named hit.

export function gather(root: string) {
  return listFiles(root);
}

const gate = { listFiles: 1 };
const { listFiles: hidden } = gate;
void hidden;
