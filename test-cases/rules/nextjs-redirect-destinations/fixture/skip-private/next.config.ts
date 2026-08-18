const nextConfig = {
  async redirects() {
    return [
      {
        source: '/hidden',
        destination: '/secret',
        permanent: true,
      }
    ];
  }
};

export default nextConfig;
