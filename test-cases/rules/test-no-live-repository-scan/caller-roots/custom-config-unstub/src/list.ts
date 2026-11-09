import { readdirSync } from "node:fs";

export function list() {
  return readdirSync(process.env.ROOT_DIR ?? "./src");
}
