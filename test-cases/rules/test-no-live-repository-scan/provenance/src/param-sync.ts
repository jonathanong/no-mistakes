import { readdirSync } from "node:fs";

const DEFAULT_ROOT = "./tmp/default-root";

export function sync(user: string, dir = DEFAULT_ROOT) {
  return readdirSync(dir).map((name) => `${user}:${name}`);
}
