// Runner discovery reads this helper before the shared fact collection.
export const setupFiles = ['./unrelated.tsx'];
export function configure(value: string) { return value; }
configure('setup');
