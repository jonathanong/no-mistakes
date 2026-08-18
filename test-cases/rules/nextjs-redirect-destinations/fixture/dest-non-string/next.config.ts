const dest = "/about";

const nextConfig = {
  async redirects() {
    return [
      {
        source: "/old",
        destination: dest,
        permanent: true,
      },
    ];
  },
};

export default nextConfig;
