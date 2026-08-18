const redirects = () => [
  {
    source: "/old",
    destination: "/gone",
    permanent: true,
  },
];

const rewrites = () => [
  {
    source: "/legacy",
    destination: "/missing",
  },
];

const nextConfig = {
  redirects,
  rewrites,
};

export default nextConfig;
