const nextConfig = {
  async redirects() {
    return [
      {
        source: '/post',
        destination: '/blog/hello',
        permanent: true,
      },
      {
        source: '/guide',
        destination: '/docs/a/b',
        permanent: true,
      },
      {
        source: '/opt',
        destination: '/optional',
        permanent: true,
      }
    ];
  }
};

export default nextConfig;
