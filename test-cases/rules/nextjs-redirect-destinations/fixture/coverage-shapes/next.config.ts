class NextConfig {
  other() {
    return [];
  }

  extra = 1;

  async redirects() {
    return [
      {
        source: "/old",
        destination: "/about",
        permanent: true,
      },
    ];
  }

  rewrites = () => [
    {
      source: "/signin",
      destination: "/login",
    },
  ];
}

export default NextConfig;
