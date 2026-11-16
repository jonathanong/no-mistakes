import { readdirSync } from "node:fs";

export function processSync(root: string) {
  return readdirSync(root);
}

const workers = { sync: processSync };
export default workers;
