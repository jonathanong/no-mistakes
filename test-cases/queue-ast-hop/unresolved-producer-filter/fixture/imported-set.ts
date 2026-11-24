import { sharedNames } from "./shared-set";

export function trackImported(name: string): void {
  sharedNames.add(name);
}
