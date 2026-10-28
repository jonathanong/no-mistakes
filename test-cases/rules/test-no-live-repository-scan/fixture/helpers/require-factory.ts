export const require = Object.assign(
  (name: string) => ({ readFileSync: () => name, readdirSync: () => [] }),
  { resolve: (name: string) => name },
);
