import { readdirSync } from "node:fs";

export function syncMixed(user: string, dir = "./tmp/mixed") {
  return readdirSync(dir).map((name) => `${user}:${name}`);
}
