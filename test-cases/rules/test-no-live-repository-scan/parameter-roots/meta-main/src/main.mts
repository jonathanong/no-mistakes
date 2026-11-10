import { readdirSync } from "node:fs";

const gate = { main: true };

if (import.meta.main) {
  void gate.main;
}

export function main(root: string) {
  readdirSync(root);
}
