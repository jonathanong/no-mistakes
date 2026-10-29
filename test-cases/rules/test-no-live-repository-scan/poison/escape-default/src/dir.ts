import { readdirSync } from "node:fs";

// The default is the fallback when a caller omits the owned directory.
export default function list(
  user: string,
  dir = "./tmp/poison",
) {
  return readdirSync(dir).map((entry) => `${user}:${entry}`);
}
