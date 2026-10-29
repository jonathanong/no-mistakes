import { readdirSync } from "node:fs";

export function syncEscaped(user: string, dir = "./tmp/escape") {
  return readdirSync(dir).map((entry) => `${user}:${entry}`);
}
