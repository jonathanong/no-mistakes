const nextConfig = {
  async redirects() {
    return [
      {
        source: '/old-about',
        destination: '/about',
        permanent: true,
      },
      {
        source: '/about-query',
        destination: '/about?ref=1#top',
        permanent: true,
      },
      {
        source: '/external',
        destination: 'https://example.com/docs',
        permanent: true,
      },
      {
        source: '/protocol-relative',
        destination: '//cdn.example.com/asset',
        permanent: true,
      },
      {
        source: '/param',
        destination: '/:slug',
        permanent: true,
      }
    ];
  }
};

export default nextConfig;
