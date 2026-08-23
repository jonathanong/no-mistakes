const nextConfig = {
  async redirects() {
    return [
      {
        source: '/old',
        destination: '/about',
        permanent: true,
      }
    ];
  }
};

export default nextConfig;
