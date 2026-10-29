import { readdirSync } from "node:fs";

const DEFAULT_DIR = "./tmp/omitted";

export function syncOmitted(user: string, dir = DEFAULT_DIR) {
  return readdirSync(dir).map((name) => `${user}:${name}`);
}
