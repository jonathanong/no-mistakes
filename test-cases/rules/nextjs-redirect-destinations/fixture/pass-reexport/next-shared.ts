const nextConfig = {
  async redirects() {
    return [{ source: "/old", destination: "/about", permanent: true }];
  },
  async rewrites() {
    return [{ source: "/alias", destination: "/about" }];
  },
};

export default nextConfig;
