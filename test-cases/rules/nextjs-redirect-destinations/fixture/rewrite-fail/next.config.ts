const nextConfig = {
  async redirects() {
    return [
      {
        source: "/old-about",
        destination: "/about",
        permanent: true,
      },
    ];
  },
  async rewrites() {
    return {
      beforeFiles: [
        {
          source: "/legacy",
          destination: "/missing-before",
        },
      ],
      afterFiles: [
        {
          source: "/legacy-after",
          destination: "/missing-after",
        },
      ],
      fallback: [
        {
          source: "/legacy-fallback",
          destination: "/missing-fallback",
        },
      ],
    };
  },
};

export default nextConfig;
