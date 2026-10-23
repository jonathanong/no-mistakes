// Keep unknown beside literals: low must preserve concrete findings under every policy.
export default {
  async redirects() {
    return [
      { source: '/old', destination: '/gone', permanent: true },
      { source: '/dynamic', destination: unknownDestination, permanent: true },
    ];
  },
  async rewrites() {
    return [
      { source: '/rewrite', destination: '/missing' },
      { source: '/dynamic-rewrite', destination: unknownDestination },
    ];
  },
};
