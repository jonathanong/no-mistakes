import { collect } from "./collect";

export const names = new Set<string>();

export function register(name: string, extra: string[]): void {
  names.add(name);
  for (const entry of collect(extra)) {
    names.add(entry);
  }
}
