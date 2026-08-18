// no-mistakes-disable-file nextjs-redirect-destinations
const nextConfig = {
  async redirects() {
    return [
      {
        source: "/old",
        destination: "/gone",
        permanent: true,
      },
    ];
  },
};

export default nextConfig;
