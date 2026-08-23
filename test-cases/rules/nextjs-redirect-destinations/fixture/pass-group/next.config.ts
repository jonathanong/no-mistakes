const nextConfig = {
  async redirects() {
    return [
      {
        source: '/signin',
        destination: '/login',
        permanent: true,
      },
      {
        source: '/prefs',
        destination: '/settings',
        permanent: true,
      }
    ];
  }
};

export default nextConfig;
