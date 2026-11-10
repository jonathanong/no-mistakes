import { readdirSync } from "node:fs";

export function scan({ opts }: { opts: { root: string } }) {
  return readdirSync(opts.root, { recursive: true });
}
