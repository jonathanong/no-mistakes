export const sharedNames = new Set<string>();

export function trackLocal(name: string): void {
  sharedNames.add(name);
}
