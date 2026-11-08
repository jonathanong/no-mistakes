import { readdirSync } from "node:fs";

export function scanOwnedArgument(root: string): string[] {
  return readdirSync(root);
}

export function scanOwnedEnvironment(): string[] {
  return readdirSync(process.env.OWNED_SCAN_ROOT ?? "");
}
