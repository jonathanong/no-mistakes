import { readdirSync } from "node:fs";

export function syncCalled(user: string, dir = "./tmp/call") {
  return readdirSync(dir).map((entry) => `${user}:${entry}`);
}
