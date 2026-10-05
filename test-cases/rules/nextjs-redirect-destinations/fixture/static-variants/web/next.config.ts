const decoy = [['old', 'decoy-missing']];
export default {
  decoy: [{ destination: '/outside-redirects' }],
  async redirects() {
    const local: readonly (readonly [string, string])[] = [['x', 'about']];
    const [prefix] = ['/'];
    const mappings = { pairs: local };
    const routes = mappings.pairs.map(([src, dst]) => {
      const destination = `${prefix}${dst}`;
      return { source: `/old/${src}`, destination };
    });
    return [...routes, ...hoisted.map(([src, dst]) => ({ source: src, destination: `/${dst}` })),
      { destination: `/${local[0][1]}` }];
  },
};
const hoisted = [['y', 'about']] as const;
