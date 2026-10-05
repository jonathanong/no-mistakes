const routePairs = [['legacy', 'missing-route']] as const;
export default {
  async redirects() {
    return [
      { source: '/home-old', destination: '/', permanent: false },
      ...routePairs.map(([src, dst]) => ({
        source: `/old/${src}`,
        destination: `/${dst}`,
        permanent: false,
      })),
    ];
  },
};
