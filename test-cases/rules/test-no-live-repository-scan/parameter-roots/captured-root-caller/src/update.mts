import { listFiles } from "./fn.mts";

export function update(root: string) {
  return listFiles(root);
}
