const nextConfig = {
  async redirects() {
    return [{ source: "/old", destination: "/gone", permanent: true }];
  },
  async rewrites() {
    return [{ source: "/alias", destination: "/missing" }];
  },
};

export default nextConfig;
