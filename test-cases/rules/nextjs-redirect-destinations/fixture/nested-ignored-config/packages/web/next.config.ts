// The synthetic Git fixture stages only the tracked routes, leaving these pages on disk.
const pairs = [
  ["/tuple-untracked", "/untracked/tuple"],
  ["/tuple-ignored", "/ignored/tuple"],
] as const;
export default {
  async redirects() {
    return [
      { source: "/old-tracked", destination: "/tracked", permanent: true },
      { source: "/old-untracked", destination: "/untracked", permanent: true },
      { source: "/old-ignored", destination: "/ignored", permanent: true },
      { source: "/old-group", destination: "/grouped", permanent: true },
      { source: "/old-post", destination: "/posts/one", permanent: true },
      { source: "/old-docs", destination: "/docs/a/b", permanent: true },
      { source: "/old-optional", destination: "/optional", permanent: true },
      ...pairs.map(([source, target]) => ({ source, destination: `${target}`, permanent: true })),
    ];
  },
  async rewrites() {
    return [
      { source: "/rewrite-untracked", destination: "/untracked" },
      { source: "/rewrite-ignored", destination: "/ignored" },
    ];
  },
};
